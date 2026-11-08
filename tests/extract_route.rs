mod common;

use axum::http::{StatusCode, header};
use common::{auth_headers, body_json, send, send_with_body, setup_env};
use jobs_api_gateway::{
    app::router,
    errors::catalog::{bad_request, unauthorized},
    features::v1::auth::jwt::sign_token,
};

// Route of the extract feature
const URI: &str = "/api/v1/admin/extractMails";

// Client headers, a json content type and a fresh bearer token
fn authed_headers() -> Vec<(&'static str, String)> {
    setup_env();

    let Ok(login) = sign_token("admin", "admin") else {
        panic!("signing a token failed");
    };

    vec![
        ("x-client-id", auth_headers()[0].1.to_string()),
        ("x-client-secret", auth_headers()[1].1.to_string()),
        ("content-type", "application/json".to_string()),
        ("authorization", format!("Bearer {}", login.token)),
    ]
}

// Posts a body with the authed headers and returns the status and the json reply
async fn post_authed(body: &str) -> (StatusCode, serde_json::Value) {
    let owned = authed_headers();
    let headers: Vec<(&str, &str)> = owned
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();

    let response = send_with_body(router(), "POST", URI, &headers, body).await;
    let status = response.status();

    (status, body_json(response).await)
}

// Without the client headers the access gate redirects to /login, the handler never runs (SPEC section 4.2)
#[tokio::test]
async fn missing_client_headers_are_redirected_to_login() {
    let response = send_with_body(router(), "POST", URI, &[], "{}").await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(response.headers()[header::LOCATION], "/login");
}

// With the client headers but without a bearer token the route answers 401
#[tokio::test]
async fn missing_bearer_token_returns_401() {
    let response = send_with_body(
        router(),
        "POST",
        URI,
        &[
            ("x-client-id", auth_headers()[0].1),
            ("x-client-secret", auth_headers()[1].1),
            ("content-type", "application/json"),
        ],
        "{}",
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = body_json(response).await;

    assert_eq!(
        body["error"],
        unauthorized::MISSING_AUTHORIZATION_TOKEN.error
    );
}

// The route only takes POST
#[tokio::test]
async fn get_returns_405() {
    let response = send(router(), "GET", URI, &auth_headers()).await;

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

// A body that is not json returns 400
#[tokio::test]
async fn invalid_json_returns_400() {
    let (status, body) = post_authed("not json").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], bad_request::INVALID_INPUT.error);
}

// Bulk with a date field returns 400 before any Gmail or database call
#[tokio::test]
async fn bulk_with_a_date_returns_400() {
    let (status, body) =
        post_authed(r#"{"type":"Bulk","query":"jobs","inputDateAfter":"2026/11/2"}"#).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "BAD_REQUEST");
}

// Increment without a date returns 400 before any Gmail or database call
#[tokio::test]
async fn increment_without_a_date_returns_400() {
    let (status, body) = post_authed(r#"{"type":"Increment","query":"jobs"}"#).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "BAD_REQUEST");
}
