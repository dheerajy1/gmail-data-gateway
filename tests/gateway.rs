use std::sync::{Arc, Mutex};

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use jobs_api_gateway::{
    clients::kafka_gateway::{GatewayPublishRequest, GatewayTarget, post_to_gateway_at},
    errors::{
        api_error::ApiError, catalog::internal_server_error::KAFKA_GATEWAY_FAILED, code::ErrorCode,
    },
};
use serde_json::{Value, json};

// Topic and correlation id used by every test
const TOPIC: &str = "test-topic";
const CORRELATION_ID: &str = "0190a3c0-7b1e-7000-8000-000000000001";

// What the fake gateway received: the headers and the json body
type Seen = Arc<Mutex<Option<(HeaderMap, Value)>>>;

// State of the fake gateway: the reply it sends and the request it saw
#[derive(Clone)]
struct Fake {
    status: StatusCode,
    content_type: &'static str,
    body: String,
    seen: Seen,
}

// Fake POST /api/v1/http-publish, records the request and sends the configured reply
async fn http_publish(State(fake): State<Fake>, headers: HeaderMap, body: Bytes) -> Response {
    let value: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

    *fake.seen.lock().unwrap() = Some((headers, value));

    (
        fake.status,
        [(header::CONTENT_TYPE, fake.content_type)],
        fake.body,
    )
        .into_response()
}

// Starts a fake gateway on a free local port and returns its base url and the request record
async fn start_fake(status: StatusCode, content_type: &'static str, body: &str) -> (String, Seen) {
    let seen: Seen = Arc::new(Mutex::new(None));

    let fake = Fake {
        status,
        content_type,
        body: body.to_string(),
        seen: seen.clone(),
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

// A json reply with the given success flag and status code in the body
fn reply(success: bool, status_code: u16) -> String {
    json!({
        "success": success,
        "statusCode": status_code,
        "message": "Event accepted",
        "data": { "topic": TOPIC, "correlationId": CORRELATION_ID },
    })
    .to_string()
}

// The request every test publishes
fn request() -> GatewayPublishRequest<Value> {
    GatewayPublishRequest {
        topic: TOPIC.to_string(),
        correlation_id: CORRELATION_ID.to_string(),
        value: json!({ "id": 1, "name": "event" }),
    }
}

// Publishes the test request to the given base url with test credentials
async fn publish(base_url: &str) -> Result<(), ApiError> {
    let target = GatewayTarget {
        base_url,
        client_id: "test-client",
        client_secret: "test-secret",
        api_key: "test-key",
        api_secret: "test-key-secret",
    };

    post_to_gateway_at(&target, &request()).await.map(|_| ())
}

// The error is the generic gateway error
fn assert_gateway_error(result: Result<(), ApiError>) {
    let error = result.unwrap_err();

    assert_eq!(error.code(), ErrorCode::InternalServerError);
    assert_eq!(error.error(), KAFKA_GATEWAY_FAILED.error);
}

// A 202 reply with success true and statusCode 202 is accepted and parsed
#[tokio::test]
async fn accepted_reply_is_parsed() {
    let (base_url, _) =
        start_fake(StatusCode::ACCEPTED, "application/json", &reply(true, 202)).await;

    let target = GatewayTarget {
        base_url: &base_url,
        client_id: "test-client",
        client_secret: "test-secret",
        api_key: "test-key",
        api_secret: "test-key-secret",
    };

    let parsed = post_to_gateway_at(&target, &request()).await.unwrap();

    assert!(parsed.success);
    assert_eq!(parsed.status_code, 202);
    assert_eq!(parsed.data.topic, TOPIC);
    assert_eq!(parsed.data.correlation_id, CORRELATION_ID);
}

// The request carries the four auth headers and the body topic, correlationId and value only
#[tokio::test]
async fn request_has_four_headers_and_three_body_fields() {
    let (base_url, seen) =
        start_fake(StatusCode::ACCEPTED, "application/json", &reply(true, 202)).await;

    publish(&base_url).await.unwrap();

    let (headers, body) = seen.lock().unwrap().take().unwrap();

    assert_eq!(headers["x-client-id"], "test-client");
    assert_eq!(headers["x-client-secret"], "test-secret");
    assert_eq!(headers["x-api-key"], "test-key");
    assert_eq!(headers["x-api-secret"], "test-key-secret");

    let object = body.as_object().unwrap();

    assert_eq!(object.len(), 3);
    assert_eq!(object["topic"], TOPIC);
    assert_eq!(object["correlationId"], CORRELATION_ID);
    assert_eq!(object["value"], json!({ "id": 1, "name": "event" }));
}

// A trailing slash on the base url does not break the path
#[tokio::test]
async fn trailing_slash_in_base_url_is_trimmed() {
    let (base_url, _) =
        start_fake(StatusCode::ACCEPTED, "application/json", &reply(true, 202)).await;

    publish(&format!("{base_url}/")).await.unwrap();
}

// A 500 reply is a publish failure
#[tokio::test]
async fn server_error_status_fails() {
    let (base_url, _) = start_fake(
        StatusCode::INTERNAL_SERVER_ERROR,
        "application/json",
        &reply(true, 202),
    )
    .await;

    assert_gateway_error(publish(&base_url).await);
}

// A 400 reply is a publish failure
#[tokio::test]
async fn client_error_status_fails() {
    let (base_url, _) = start_fake(
        StatusCode::BAD_REQUEST,
        "application/json",
        &reply(false, 400),
    )
    .await;

    assert_gateway_error(publish(&base_url).await);
}

// A 2xx reply that is not json is a publish failure
#[tokio::test]
async fn non_json_body_fails() {
    let (base_url, _) = start_fake(StatusCode::ACCEPTED, "text/plain", "accepted").await;

    assert_gateway_error(publish(&base_url).await);
}

// A json reply without the expected fields is a publish failure
#[tokio::test]
async fn reply_without_expected_fields_fails() {
    let (base_url, _) =
        start_fake(StatusCode::ACCEPTED, "application/json", r#"{"ok":true}"#).await;

    assert_gateway_error(publish(&base_url).await);
}

// A reply whose statusCode is not 202 is a publish failure (D21)
#[tokio::test]
async fn wrong_status_code_in_body_fails() {
    let (base_url, _) =
        start_fake(StatusCode::ACCEPTED, "application/json", &reply(true, 200)).await;

    assert_gateway_error(publish(&base_url).await);
}

// A reply whose success is false is a publish failure
#[tokio::test]
async fn success_false_in_body_fails() {
    let (base_url, _) =
        start_fake(StatusCode::ACCEPTED, "application/json", &reply(false, 202)).await;

    assert_gateway_error(publish(&base_url).await);
}

// A gateway that cannot be reached is a publish failure
#[tokio::test]
async fn unreachable_gateway_fails() {
    assert_gateway_error(publish("http://127.0.0.1:9").await);
}
