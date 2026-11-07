mod common;

use axum::{
    body::Body,
    http::{Response, StatusCode, header},
};
use common::{auth_headers, body_json, send};
use jobs_api_gateway::app::router;

// Sends a GET request with the given headers through the real router
async fn get(uri: &str, headers: &[(&str, &str)]) -> Response<Body> {
    send(router(), "GET", uri, headers).await
}

// Reads the Location header of a response as text
fn location(response: &Response<Body>) -> Option<&str> {
    response
        .headers()
        .get(header::LOCATION)
        .and_then(|value| value.to_str().ok())
}

// A browser without the cookie is redirected to /login
#[tokio::test]
async fn browser_without_cookie_is_redirected_to_login() {
    let response = get("/healthz/app", &[("accept", "text/html")]).await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(location(&response), Some("/login"));
}

// A browser with the auth cookie reaches the route
#[tokio::test]
async fn browser_with_cookie_reaches_the_route() {
    let response = get(
        "/healthz/app",
        &[("accept", "text/html"), ("cookie", "auth=ok")],
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
}

// A browser with a wrong cookie value is redirected to /login
#[tokio::test]
async fn browser_with_wrong_cookie_is_redirected_to_login() {
    let response = get(
        "/healthz/app",
        &[("accept", "text/html"), ("cookie", "auth=no")],
    )
    .await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(location(&response), Some("/login"));
}

// An html request with valid client headers but no cookie is still redirected
#[tokio::test]
async fn html_request_with_client_headers_is_redirected() {
    let mut headers = vec![("accept", "text/html")];
    headers.extend(auth_headers());

    let response = get("/healthz/app", &headers).await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(location(&response), Some("/login"));
}

// A non-html request with valid client headers passes without a cookie
#[tokio::test]
async fn api_request_with_client_headers_passes_without_cookie() {
    let response = get("/healthz/app", &auth_headers()).await;

    assert_eq!(response.status(), StatusCode::OK);
}

// A non-html request with a wrong secret and no cookie is redirected
#[tokio::test]
async fn api_request_with_wrong_secret_is_redirected() {
    let response = get(
        "/healthz/app",
        &[("x-client-id", "test-client"), ("x-client-secret", "wrong")],
    )
    .await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(location(&response), Some("/login"));
}

// A non-html request without any headers is redirected
#[tokio::test]
async fn api_request_without_headers_is_redirected() {
    let response = get("/healthz/app", &[]).await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(location(&response), Some("/login"));
}

// The auth cookie also lets a non-html request through
#[tokio::test]
async fn api_request_with_cookie_passes() {
    let response = get("/healthz/app", &[("cookie", "auth=ok")]).await;

    assert_eq!(response.status(), StatusCode::OK);
}

// The login page is open without a cookie
#[tokio::test]
async fn login_page_is_open_without_cookie() {
    let response = get("/login", &[("accept", "text/html")]).await;

    assert_eq!(response.status(), StatusCode::OK);
}

// Static files are open without a cookie
#[tokio::test]
async fn static_files_are_open_without_cookie() {
    let response = get("/global.css", &[("accept", "text/html")]).await;

    assert_eq!(response.status(), StatusCode::OK);
}

// An unknown path does not go through the gate and returns the 404 error shape
#[tokio::test]
async fn unknown_path_is_not_gated() {
    let response = get("/missing", &[("accept", "text/html")]).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = body_json(response).await;

    assert_eq!(body["code"], "NOT_FOUND");
}
