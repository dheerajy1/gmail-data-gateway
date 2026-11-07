mod common;

use axum::http::{StatusCode, header};
use common::{body_json, send};

// GET /api/v1/health returns the health envelope with the app data
#[tokio::test]
async fn health_returns_200_with_app_data() {
    let response = send("GET", "/api/v1/health").await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;

    assert_eq!(body["success"], true);
    assert_eq!(body["statusCode"], 200);
    assert_eq!(body["message"], "Health check OK");
    assert_eq!(body["data"]["app"]["status"], "UP");
    assert!(body["data"]["app"]["version"].is_string());
    assert!(body["data"]["app"]["uptimeMs"].is_u64());
    assert!(body["data"]["app"]["timestampUtc"].is_string());
}

// The stylesheet is served without authentication as css
#[tokio::test]
async fn global_css_is_served() {
    let response = send("GET", "/global.css").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/css; charset=utf-8"
    );
}

// The logo is served without authentication as png
#[tokio::test]
async fn logo_is_served() {
    let response = send("GET", "/logo.png").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "image/png");
}

// The favicon is served without authentication as an icon
#[tokio::test]
async fn favicon_is_served() {
    let response = send("GET", "/favicon.ico").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "image/x-icon");
}
