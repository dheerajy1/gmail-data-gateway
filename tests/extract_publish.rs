use std::sync::{Arc, Mutex};

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use jobs_api_gateway::{
    clients::kafka_gateway::GatewayTarget,
    errors::{catalog::internal_server_error::KAFKA_GATEWAY_FAILED, code::ErrorCode},
    features::v1::extract_mails::{
        model::ExtractType, publisher::PublishConfig, reader::MailRecord, service::publish_extract,
    },
};
use serde_json::{Value, json};
use uuid::Uuid;

// Values the tests put into the events
const CONFIG: PublishConfig<'static> = PublishConfig {
    topic: "write-topic",
    write_model: "write-procedure",
    read_model: "read-procedure",
    source_client_id: "source-1",
};

// State of the fake gateway: the requests it saw and the call it fails on, counted from 1
#[derive(Clone)]
struct Fake {
    seen: Arc<Mutex<Vec<Value>>>,
    fail_on_call: Option<usize>,
}

// Fake POST /api/v1/http-publish, records the request and replies 202 like the real gateway
async fn http_publish(State(fake): State<Fake>, Json(body): Json<Value>) -> impl IntoResponse {
    let call = {
        let mut seen = fake.seen.lock().unwrap();

        seen.push(body.clone());

        seen.len()
    };

    if fake.fail_on_call == Some(call) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "boom" })),
        );
    }

    (
        StatusCode::ACCEPTED,
        Json(json!({
            "success": true,
            "statusCode": 202,
            "message": "accepted",
            "data": { "topic": body["topic"], "correlationId": body["correlationId"] },
        })),
    )
}

// Starts a fake gateway on a free local port and returns its base url and the request record
async fn start_fake(fail_on_call: Option<usize>) -> (String, Arc<Mutex<Vec<Value>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));

    let fake = Fake {
        seen: seen.clone(),
        fail_on_call,
    };

    let router = Router::new()
        .route("/api/v1/http-publish", post(http_publish))
        .with_state(fake);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    (format!("http://{address}"), seen)
}

// Builds a target that points at the fake gateway
fn target(base_url: &str) -> GatewayTarget<'_> {
    GatewayTarget {
        base_url,
        client_id: "client",
        client_secret: "secret",
        api_key: "key",
        api_secret: "key-secret",
    }
}

// Builds mails like Gmail lists them: newest first, so mail-<count> comes first and mail-1 last
fn mails_newest_first(count: usize) -> Vec<MailRecord> {
    (1..=count)
        .rev()
        .map(|n| MailRecord {
            id: format!("mail-{n}"),
            date: "2026-11-02T15:45:30+05:30".to_string(),
            from: "Sender <sender@example.com>".to_string(),
            subject: format!("Subject {n}"),
            snippet: format!("Snippet {n}"),
        })
        .collect()
}

// Number of records in a recorded request
fn record_count(request: &Value) -> usize {
    request["value"]["records"].as_array().map_or(0, Vec::len)
}

// Id of the record at a position in a recorded request
fn record_id(request: &Value, index: usize) -> String {
    request["value"]["records"][index]["id"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

// 250 mails become three events of 100, 100 and 50 records, oldest first, with UUIDv7 correlation ids
#[tokio::test]
async fn publishes_250_mails_as_three_chunks_oldest_first() {
    let (base_url, seen) = start_fake(None).await;

    let Ok(outcome) = publish_extract(
        ExtractType::Bulk,
        mails_newest_first(250),
        &CONFIG,
        &target(&base_url),
    )
    .await
    else {
        panic!("publishing failed");
    };

    let requests = seen.lock().unwrap().clone();

    assert_eq!(requests.len(), 3);
    assert_eq!(
        requests.iter().map(record_count).collect::<Vec<_>>(),
        [100, 100, 50]
    );

    assert_eq!(record_id(&requests[0], 0), "mail-1");
    assert_eq!(record_id(&requests[1], 0), "mail-101");
    assert_eq!(record_id(&requests[2], 0), "mail-201");
    assert_eq!(record_id(&requests[2], 49), "mail-250");

    for (index, request) in requests.iter().enumerate() {
        let correlation_id = request["correlationId"].as_str().unwrap_or_default();

        assert_eq!(request["topic"], "write-topic");
        assert_eq!(request["value"]["correlationId"], correlation_id);
        assert_eq!(request["value"]["job"]["writeModel"], "write-procedure");
        assert_eq!(request["value"]["job"]["readModel"], "read-procedure");
        assert_eq!(request["value"]["sourceClientId"], "source-1");
        assert_eq!(outcome.data.correlation_ids[index], correlation_id);

        let Ok(parsed) = Uuid::parse_str(correlation_id) else {
            panic!("correlation id is not a UUID");
        };

        assert_eq!(parsed.get_version_num(), 7);
    }

    assert_eq!(outcome.data.record_count, 250);
    assert_eq!(outcome.data.chunk_count, 3);
    assert_eq!(outcome.data.status, "ACCEPTED");
    assert!(outcome.message.starts_with("Bulk accepted."));
    assert!(outcome.message.contains("asynchronous"));
}

// Exactly 100 mails make one chunk, 101 make two
#[tokio::test]
async fn chunk_boundaries_are_100_and_101() {
    let (base_url, seen) = start_fake(None).await;

    let Ok(outcome) = publish_extract(
        ExtractType::Sync,
        mails_newest_first(100),
        &CONFIG,
        &target(&base_url),
    )
    .await
    else {
        panic!("publishing failed");
    };

    assert_eq!(outcome.data.chunk_count, 1);
    assert_eq!(seen.lock().unwrap().len(), 1);

    let (base_url, seen) = start_fake(None).await;

    let Ok(outcome) = publish_extract(
        ExtractType::Sync,
        mails_newest_first(101),
        &CONFIG,
        &target(&base_url),
    )
    .await
    else {
        panic!("publishing failed");
    };

    let requests = seen.lock().unwrap().clone();

    assert_eq!(outcome.data.chunk_count, 2);
    assert_eq!(
        requests.iter().map(record_count).collect::<Vec<_>>(),
        [100, 1]
    );
}

// No mails publish nothing and answer with NO_NEW_RECORDS
#[tokio::test]
async fn no_mails_publish_nothing() {
    let (base_url, seen) = start_fake(None).await;

    let Ok(outcome) = publish_extract(
        ExtractType::Increment,
        Vec::new(),
        &CONFIG,
        &target(&base_url),
    )
    .await
    else {
        panic!("publishing failed");
    };

    assert!(seen.lock().unwrap().is_empty());
    assert!(outcome.data.correlation_ids.is_empty());
    assert_eq!(outcome.data.record_count, 0);
    assert_eq!(outcome.data.chunk_count, 0);
    assert_eq!(outcome.data.status, "NO_NEW_RECORDS");
}

// A failed chunk stops the run and the error says how many chunks were already published
#[tokio::test]
async fn failed_chunk_stops_the_run_and_reports_progress() {
    let (base_url, seen) = start_fake(Some(2)).await;

    let Err(error) = publish_extract(
        ExtractType::Bulk,
        mails_newest_first(250),
        &CONFIG,
        &target(&base_url),
    )
    .await
    else {
        panic!("publishing should have failed");
    };

    assert_eq!(error.code(), ErrorCode::InternalServerError);
    assert_eq!(error.error(), KAFKA_GATEWAY_FAILED.error);
    assert!(error.message().starts_with(KAFKA_GATEWAY_FAILED.message));
    assert!(
        error
            .message()
            .contains("Chunks already published: 1 of 3.")
    );
    assert_eq!(seen.lock().unwrap().len(), 2);
}
