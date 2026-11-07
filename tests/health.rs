mod common;

use axum::{
    body::Body,
    http::{Response, StatusCode, header},
};
use common::{auth_headers, body_json, body_text, send};
use jobs_api_gateway::app::router;

// Sends a GET request with the given headers through the real router
async fn get(uri: &str, headers: &[(&str, &str)]) -> Response<Body> {
    send(router(), "GET", uri, headers).await
}

// GET /api/v1/health returns 503 while the database is down, the test database does not exist
#[tokio::test]
async fn health_returns_503_while_db_is_down() {
    let response = get("/api/v1/health", &auth_headers()).await;

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let body = body_json(response).await;

    assert_eq!(body["success"], false);
    assert_eq!(body["statusCode"], 503);
    assert_eq!(body["code"], "SERVICE_UNAVAILABLE");
}

// GET /healthz/app returns the app health envelope as json for api clients
#[tokio::test]
async fn healthz_app_returns_json_for_api_clients() {
    let response = get("/healthz/app", &auth_headers()).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;

    assert_eq!(body["success"], true);
    assert_eq!(body["statusCode"], 200);
    assert_eq!(body["message"], "Application is running");
    assert_eq!(body["data"]["status"], "UP");
    assert!(body["data"]["version"].is_string());
    assert!(body["data"]["uptimeMs"].is_u64());
    assert!(body["data"]["timestampUtc"].is_string());
}

// GET /healthz/app returns html for a logged in browser
#[tokio::test]
async fn healthz_app_returns_html_for_browsers() {
    let response = get(
        "/healthz/app",
        &[("accept", "text/html"), ("cookie", "auth=ok")],
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);

    let text = body_text(response).await;

    assert!(text.starts_with("<!DOCTYPE html>"));
    assert!(text.contains("App Health"));
}

// GET /healthz/db reports the database as down in json while it is unreachable
#[tokio::test]
async fn healthz_db_reports_down_in_json() {
    let response = get("/healthz/db", &auth_headers()).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;

    assert_eq!(body["message"], "Database connection failed");
    assert_eq!(body["data"]["status"], "DOWN");
    assert_eq!(body["data"]["database"], "DISCONNECTED");
    assert!(body["data"]["latencyMs"].is_null());
}

// GET /healthz/db returns html for a logged in browser
#[tokio::test]
async fn healthz_db_returns_html_for_browsers() {
    let response = get(
        "/healthz/db",
        &[("accept", "text/html"), ("cookie", "auth=ok")],
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);

    let text = body_text(response).await;

    assert!(text.contains("Database Health"));
    assert!(text.contains("DISCONNECTED"));
}

// GET /healthz returns the combined health data as json
#[tokio::test]
async fn healthz_returns_app_and_db_in_json() {
    let response = get("/healthz", &auth_headers()).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;

    assert_eq!(body["data"]["app"]["status"], "UP");
    assert_eq!(body["data"]["db"]["status"], "DOWN");
}

// The stylesheet is served without authentication as css
#[tokio::test]
async fn global_css_is_served() {
    let response = get("/global.css", &[]).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/css; charset=utf-8"
    );
}

// The logo is served without authentication as png
#[tokio::test]
async fn logo_is_served() {
    let response = get("/logo.png", &[]).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "image/png");
}

// The favicon is served without authentication as an icon
#[tokio::test]
async fn favicon_is_served() {
    let response = get("/favicon.ico", &[]).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "image/x-icon");
}
