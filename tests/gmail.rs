use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use jobs_api_gateway::{
    clients::gmail::GmailSession,
    db::google_oauth::GoogleCreds,
    errors::{
        catalog::{CatalogEntry, forbidden, too_many_requests, unauthorized},
        code::ErrorCode,
    },
};
use serde_json::json;

// State of the fake Google: what it replies and what it saw
#[derive(Clone)]
struct Fake {
    token_status: StatusCode,
    expires_in: u64,
    gmail_statuses: Arc<Mutex<VecDeque<StatusCode>>>,
    token_calls: Arc<AtomicUsize>,
    token_forms: Arc<Mutex<Vec<String>>>,
    seen_auth: Arc<Mutex<Vec<String>>>,
}

// Fake token endpoint, numbers the tokens token-1, token-2 and so on
async fn token(State(fake): State<Fake>, body: String) -> Response {
    let number = fake.token_calls.fetch_add(1, Ordering::SeqCst) + 1;

    fake.token_forms.lock().unwrap().push(body);

    let reply = if fake.token_status.is_success() {
        json!({
            "access_token": format!("token-{number}"),
            "expires_in": fake.expires_in,
            "token_type": "Bearer",
        })
    } else {
        json!({ "error": "invalid_grant" })
    };

    (
        fake.token_status,
        [(header::CONTENT_TYPE, "application/json")],
        reply.to_string(),
    )
        .into_response()
}

// Fake Gmail list endpoint, replies with the next queued status and 200 when the queue is empty
async fn messages(State(fake): State<Fake>, headers: HeaderMap) -> Response {
    let auth = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_string();

    fake.seen_auth.lock().unwrap().push(auth);

    let status = fake
        .gmail_statuses
        .lock()
        .unwrap()
        .pop_front()
        .unwrap_or(StatusCode::OK);

    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"messages":[]}"#,
    )
        .into_response()
}

// Credentials used by every test
fn creds() -> GoogleCreds {
    GoogleCreds {
        client_id: "id-1".to_string(),
        client_secret: "secret-1".to_string(),
        refresh_token: "refresh-1".to_string(),
    }
}

// Starts a fake Google on a free local port and returns it with its base url
async fn start_fake(
    token_status: StatusCode,
    expires_in: u64,
    gmail_statuses: &[StatusCode],
) -> (Fake, String) {
    let fake = Fake {
        token_status,
        expires_in,
        gmail_statuses: Arc::new(Mutex::new(gmail_statuses.iter().copied().collect())),
        token_calls: Arc::new(AtomicUsize::new(0)),
        token_forms: Arc::new(Mutex::new(Vec::new())),
        seen_auth: Arc::new(Mutex::new(Vec::new())),
    };

    let router = Router::new()
        .route("/token", post(token))
        .route("/users/me/messages", get(messages))
        .with_state(fake.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    (fake, format!("http://{address}"))
}

// Starts a fake Google and returns it with a session that points at it
async fn start(
    token_status: StatusCode,
    expires_in: u64,
    gmail_statuses: &[StatusCode],
) -> (Fake, GmailSession) {
    let (fake, base) = start_fake(token_status, expires_in, gmail_statuses).await;
    let session = GmailSession::new(
        &format!("{base}/token"),
        &format!("{base}/users/me"),
        creds(),
    );

    (fake, session)
}

// The error has the code and error text of the catalogue entry
fn assert_entry(
    error: &jobs_api_gateway::errors::api_error::ApiError,
    code: ErrorCode,
    entry: CatalogEntry,
) {
    assert_eq!(error.code(), code);
    assert_eq!(error.error(), entry.error);
}

// The token request carries the four form fields and the Gmail call carries the bearer token
#[tokio::test]
async fn token_request_and_bearer_header() {
    let (fake, session) = start(StatusCode::OK, 3599, &[]).await;

    let response = session.get("messages", &[("q", "jobs")]).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let forms = fake.token_forms.lock().unwrap();

    assert_eq!(forms.len(), 1);
    assert!(forms[0].contains("grant_type=refresh_token"));
    assert!(forms[0].contains("client_id=id-1"));
    assert!(forms[0].contains("client_secret=secret-1"));
    assert!(forms[0].contains("refresh_token=refresh-1"));
    assert_eq!(*fake.seen_auth.lock().unwrap(), ["Bearer token-1"]);
}

// A token that is still valid is reused
#[tokio::test]
async fn token_is_cached() {
    let (fake, session) = start(StatusCode::OK, 3599, &[]).await;

    session.get("messages", &[]).await.unwrap();
    session.get("messages", &[]).await.unwrap();

    assert_eq!(fake.token_calls.load(Ordering::SeqCst), 1);
}

// A token that expires within the margin is refreshed before the next call
#[tokio::test]
async fn token_close_to_expiry_is_refreshed() {
    let (fake, session) = start(StatusCode::OK, 30, &[]).await;

    session.get("messages", &[]).await.unwrap();
    session.get("messages", &[]).await.unwrap();

    assert_eq!(fake.token_calls.load(Ordering::SeqCst), 2);
}

// A 401 from Gmail refreshes the token once and retries the call
#[tokio::test]
async fn unauthorized_refreshes_once_and_retries() {
    let (fake, session) = start(StatusCode::OK, 3599, &[StatusCode::UNAUTHORIZED]).await;

    let response = session.get("messages", &[]).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(fake.token_calls.load(Ordering::SeqCst), 2);
    assert_eq!(
        *fake.seen_auth.lock().unwrap(),
        ["Bearer token-1", "Bearer token-2"]
    );
}

// A second 401 after the refresh is a session expired error and is not retried again
#[tokio::test]
async fn second_unauthorized_is_session_expired() {
    let (fake, session) = start(
        StatusCode::OK,
        3599,
        &[StatusCode::UNAUTHORIZED, StatusCode::UNAUTHORIZED],
    )
    .await;

    let error = session.get("messages", &[]).await.unwrap_err();

    assert_entry(
        &error,
        ErrorCode::Unauthorized,
        unauthorized::GMAIL_SESSION_EXPIRED,
    );
    assert_eq!(fake.seen_auth.lock().unwrap().len(), 2);
    assert_eq!(fake.token_calls.load(Ordering::SeqCst), 2);
}

// A 400 from Gmail is a session expired error without a retry
#[tokio::test]
async fn bad_request_is_session_expired() {
    let (fake, session) = start(StatusCode::OK, 3599, &[StatusCode::BAD_REQUEST]).await;

    let error = session.get("messages", &[]).await.unwrap_err();

    assert_entry(
        &error,
        ErrorCode::Unauthorized,
        unauthorized::GMAIL_SESSION_EXPIRED,
    );
    assert_eq!(fake.seen_auth.lock().unwrap().len(), 1);
}

// A 403 from Gmail is access denied
#[tokio::test]
async fn forbidden_is_access_denied() {
    let (_, session) = start(StatusCode::OK, 3599, &[StatusCode::FORBIDDEN]).await;

    let error = session.get("messages", &[]).await.unwrap_err();

    assert_entry(&error, ErrorCode::Forbidden, forbidden::GMAIL_ACCESS_DENIED);
}

// A 429 from Gmail is the rate limit error
#[tokio::test]
async fn too_many_requests_is_rate_limit() {
    let (_, session) = start(StatusCode::OK, 3599, &[StatusCode::TOO_MANY_REQUESTS]).await;

    let error = session.get("messages", &[]).await.unwrap_err();

    assert_entry(
        &error,
        ErrorCode::TooManyRequests,
        too_many_requests::GMAIL_RATE_LIMIT,
    );
}

// Any other Gmail status is a 500 with the status in the error text
#[tokio::test]
async fn other_statuses_are_internal_errors_with_the_status() {
    for status in [
        StatusCode::INTERNAL_SERVER_ERROR,
        StatusCode::SERVICE_UNAVAILABLE,
    ] {
        let (_, session) = start(StatusCode::OK, 3599, &[status]).await;

        let error = session.get("messages", &[]).await.unwrap_err();

        assert_eq!(error.code(), ErrorCode::InternalServerError);
        assert_eq!(
            error.error(),
            format!("Gmail API Error ({})", status.as_u16())
        );
    }
}

// A token endpoint that rejects the refresh token is a session expired error and Gmail is not called
#[tokio::test]
async fn rejected_refresh_token_is_session_expired() {
    let (fake, session) = start(StatusCode::BAD_REQUEST, 3599, &[]).await;

    let error = session.get("messages", &[]).await.unwrap_err();

    assert_entry(
        &error,
        ErrorCode::Unauthorized,
        unauthorized::GMAIL_SESSION_EXPIRED,
    );
    assert!(fake.seen_auth.lock().unwrap().is_empty());
}

// A token endpoint that fails with 500 is a 500 with the status in the error text
#[tokio::test]
async fn token_endpoint_failure_is_an_internal_error() {
    let (_, session) = start(StatusCode::INTERNAL_SERVER_ERROR, 3599, &[]).await;

    let error = session.get("messages", &[]).await.unwrap_err();

    assert_eq!(error.code(), ErrorCode::InternalServerError);
    assert_eq!(error.error(), "Gmail API Error (500)");
}

// A token endpoint that cannot be reached is a 500
#[tokio::test]
async fn unreachable_token_endpoint_is_an_internal_error() {
    let session = GmailSession::new("http://127.0.0.1:9/token", "http://127.0.0.1:9", creds());

    let error = session.get("messages", &[]).await.unwrap_err();

    assert_eq!(error.code(), ErrorCode::InternalServerError);
}

// A slash at the end of the base url and at the start of the path do not break the url
#[tokio::test]
async fn slashes_in_the_url_are_normalised() {
    let (fake, base) = start_fake(StatusCode::OK, 3599, &[]).await;
    let session = GmailSession::new(
        &format!("{base}/token"),
        &format!("{base}/users/me/"),
        creds(),
    );

    let response = session.get("/messages", &[]).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(fake.token_calls.load(Ordering::SeqCst), 1);
}
