use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use axum::{
    Json, Router,
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use jobs_api_gateway::{
    clients::gmail::GmailSession,
    db::google_oauth::GoogleCreds,
    errors::{catalog::too_many_requests::GMAIL_RATE_LIMIT, code::ErrorCode},
    features::v1::extract_mails::reader::{MailRecord, ReadPacing, read_mails},
};
use serde_json::{Value, json};

// No pauses in tests
const NO_PAUSE: ReadPacing = ReadPacing {
    page_delay: Duration::ZERO,
    group_delay: Duration::ZERO,
};

// State of the fake Gmail: the pages it lists, what it replies and what it saw
#[derive(Clone)]
struct Fake {
    pages: Arc<Vec<Vec<String>>>,
    list_status: StatusCode,
    list_calls: Arc<AtomicUsize>,
    get_calls: Arc<AtomicUsize>,
    list_queries: Arc<Mutex<Vec<String>>>,
    get_queries: Arc<Mutex<Vec<String>>>,
}

// Fake token endpoint, always hands out the same token
async fn token() -> Json<Value> {
    Json(json!({
        "access_token": "token-1",
        "expires_in": 3600,
        "token_type": "Bearer",
    }))
}

// Reads the page number out of a pageToken like p2, page 0 when there is none
fn page_index(raw_query: &str) -> usize {
    raw_query
        .split('&')
        .find_map(|pair| pair.strip_prefix("pageToken=p"))
        .and_then(|number| number.parse().ok())
        .unwrap_or(0)
}

// Fake message list, one page per call, the next page token is p1, p2 and so on
async fn list(State(fake): State<Fake>, RawQuery(raw): RawQuery) -> Response {
    fake.list_calls.fetch_add(1, Ordering::SeqCst);

    let raw = raw.unwrap_or_default();
    let index = page_index(&raw);

    fake.list_queries.lock().unwrap().push(raw);

    if !fake.list_status.is_success() {
        return (fake.list_status, Json(json!({ "error": "failed" }))).into_response();
    }

    let ids = fake.pages.get(index).cloned().unwrap_or_default();

    let mut body = json!({});

    if !ids.is_empty() {
        body["messages"] = ids
            .iter()
            .map(|id| json!({ "id": id, "threadId": id }))
            .collect();
    }

    if index + 1 < fake.pages.len() {
        body["nextPageToken"] = json!(format!("p{}", index + 1));
    }

    Json(body).into_response()
}

// Fake message get, a few special ids give broken mails
async fn message(
    State(fake): State<Fake>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> Response {
    fake.get_calls.fetch_add(1, Ordering::SeqCst);
    fake.get_queries
        .lock()
        .unwrap()
        .push(raw.unwrap_or_default());

    if id == "missing" {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": "gone" }))).into_response();
    }

    let date = if id == "bad-date" {
        "not a date"
    } else {
        "Mon, 02 Nov 2026 10:15:30 +0000 (UTC)"
    };

    let subject = if id == "dirty" {
        "  Hello\u{200b}   world \n".to_string()
    } else {
        format!("Subject {id}")
    };

    let mut headers = vec![
        json!({ "name": "Date", "value": date }),
        json!({ "name": "From", "value": "Sender <sender@example.com>" }),
    ];

    if id != "no-subject" {
        headers.push(json!({ "name": "Subject", "value": subject }));
    }

    Json(json!({
        "id": id,
        "snippet": format!("Snippet {id}"),
        "payload": { "headers": headers },
    }))
    .into_response()
}

// Starts a fake Gmail on a free local port and returns its state with a session pointed at it
async fn start_fake(
    pages: Vec<Vec<String>>,
    list_status: StatusCode,
) -> (Fake, &'static GmailSession) {
    let fake = Fake {
        pages: Arc::new(pages),
        list_status,
        list_calls: Arc::new(AtomicUsize::new(0)),
        get_calls: Arc::new(AtomicUsize::new(0)),
        list_queries: Arc::new(Mutex::new(Vec::new())),
        get_queries: Arc::new(Mutex::new(Vec::new())),
    };

    let router = Router::new()
        .route("/token", post(token))
        .route("/users/me/messages", get(list))
        .route("/users/me/messages/{id}", get(message))
        .with_state(fake.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    let creds = GoogleCreds {
        client_id: "id-1".to_string(),
        client_secret: "secret-1".to_string(),
        refresh_token: "refresh-1".to_string(),
    };

    // The reader needs a session that lives for the whole program, so the test leaks it
    let session = Box::leak(Box::new(GmailSession::new(
        &format!("http://{address}/token"),
        &format!("http://{address}/users/me"),
        creds,
    )));

    (fake, session)
}

// Builds ids like m1, m2 up to the count
fn ids(count: usize) -> Vec<String> {
    (1..=count).map(|n| format!("m{n}")).collect()
}

// Ids of the mails, in order
fn mail_ids(mails: &[MailRecord]) -> Vec<&str> {
    mails.iter().map(|mail| mail.id.as_str()).collect()
}

// Mails of all pages are read in order, unreadable and incomplete ones are dropped
#[tokio::test]
async fn reads_all_pages_in_order_and_drops_bad_mails() {
    let pages = vec![
        vec![
            "m1".to_string(),
            "bad-date".to_string(),
            "m2".to_string(),
            "missing".to_string(),
            "no-subject".to_string(),
            "m3".to_string(),
        ],
        vec!["m4".to_string()],
    ];

    let (fake, session) = start_fake(pages, StatusCode::OK).await;

    let Ok(mails) = read_mails(session, "jobs", None, NO_PAUSE).await else {
        panic!("reading failed");
    };

    assert_eq!(mail_ids(&mails), ["m1", "m2", "m3", "m4"]);
    assert_eq!(fake.list_calls.load(Ordering::SeqCst), 2);
    assert_eq!(fake.get_calls.load(Ordering::SeqCst), 7);
}

// A mail is normalized: India time, cleaned sender, subject and snippet
#[tokio::test]
async fn mail_fields_are_normalized() {
    let (_fake, session) = start_fake(vec![ids(1)], StatusCode::OK).await;

    let Ok(mails) = read_mails(session, "jobs", None, NO_PAUSE).await else {
        panic!("reading failed");
    };

    assert_eq!(
        mails,
        vec![MailRecord {
            id: "m1".to_string(),
            date: "2026-11-02T15:45:30+05:30".to_string(),
            from: "Sender <sender@example.com>".to_string(),
            subject: "Subject m1".to_string(),
            snippet: "Snippet m1".to_string(),
        }]
    );
}

// Invisible characters and extra whitespace are cleaned out of the headers
#[tokio::test]
async fn header_text_is_cleaned() {
    let (_fake, session) = start_fake(vec![vec!["dirty".to_string()]], StatusCode::OK).await;

    let Ok(mails) = read_mails(session, "jobs", None, NO_PAUSE).await else {
        panic!("reading failed");
    };

    assert_eq!(mails[0].subject, "Hello world");
}

// The list call sends the query, the page size and the page token, the get call asks for metadata headers
#[tokio::test]
async fn requests_carry_the_expected_parameters() {
    let (fake, session) = start_fake(vec![ids(1), ids(1)], StatusCode::OK).await;

    let Ok(_) = read_mails(session, "jobs", None, NO_PAUSE).await else {
        panic!("reading failed");
    };

    let list_queries = fake.list_queries.lock().unwrap().clone();

    assert!(list_queries[0].contains("q=jobs"));
    assert!(list_queries[0].contains("maxResults=100"));
    assert!(!list_queries[0].contains("pageToken"));
    assert!(list_queries[1].contains("pageToken=p1"));

    let get_query = fake.get_queries.lock().unwrap()[0].clone();

    assert!(get_query.contains("format=metadata"));
    assert!(get_query.contains("metadataHeaders=Date"));
    assert!(get_query.contains("metadataHeaders=From"));
    assert!(get_query.contains("metadataHeaders=Subject"));
}

// With a limit the run stops after the first group that reaches it and cuts the list
#[tokio::test]
async fn limit_stops_after_the_first_group() {
    let (fake, session) = start_fake(vec![ids(45), ids(5)], StatusCode::OK).await;

    let Ok(mails) = read_mails(session, "jobs", Some(5), NO_PAUSE).await else {
        panic!("reading failed");
    };

    assert_eq!(mail_ids(&mails), ["m1", "m2", "m3", "m4", "m5"]);
    assert_eq!(fake.list_calls.load(Ordering::SeqCst), 1);
    assert_eq!(fake.get_calls.load(Ordering::SeqCst), 20);
}

// A limit larger than the first page continues to the next page and still cuts the list
#[tokio::test]
async fn limit_can_span_pages() {
    let (fake, session) = start_fake(vec![ids(3), ids(3)], StatusCode::OK).await;

    let Ok(mails) = read_mails(session, "jobs", Some(4), NO_PAUSE).await else {
        panic!("reading failed");
    };

    assert_eq!(mails.len(), 4);
    assert_eq!(fake.list_calls.load(Ordering::SeqCst), 2);
    assert_eq!(fake.get_calls.load(Ordering::SeqCst), 6);
}

// A query without messages gives an empty list and reads nothing
#[tokio::test]
async fn no_messages_gives_an_empty_list() {
    let (fake, session) = start_fake(vec![vec![]], StatusCode::OK).await;

    let Ok(mails) = read_mails(session, "jobs", None, NO_PAUSE).await else {
        panic!("reading failed");
    };

    assert!(mails.is_empty());
    assert_eq!(fake.get_calls.load(Ordering::SeqCst), 0);
}

// A Gmail failure on the list call fails the run with the mapped error
#[tokio::test]
async fn list_failure_is_mapped_to_an_api_error() {
    let (_fake, session) = start_fake(vec![ids(1)], StatusCode::TOO_MANY_REQUESTS).await;

    let Err(error) = read_mails(session, "jobs", None, NO_PAUSE).await else {
        panic!("reading should have failed");
    };

    assert_eq!(error.code(), ErrorCode::TooManyRequests);
    assert_eq!(error.error(), GMAIL_RATE_LIMIT.error);
}
