mod common;

use axum::{
    body::Body,
    http::{Response, StatusCode},
};
use common::{auth_headers, body_json, send};
use http_body_util::BodyExt;
use jobs_api_gateway::app::router;
use serde_json::Value;

// Sends a GET request with the given headers through the real router
async fn get(uri: &str, headers: &[(&str, &str)]) -> Response<Body> {
    send(router(), "GET", uri, headers).await
}

// Fetches the spec from GET /openapi/json
async fn spec() -> Value {
    let response = get("/openapi/json", &auth_headers()).await;

    assert_eq!(response.status(), StatusCode::OK);

    body_json(response).await
}

// The spec carries the title and version of the old docs page
#[tokio::test]
async fn spec_has_title_and_version() {
    let spec = spec().await;

    assert_eq!(spec["info"]["title"], "Gmail data gateway API");
    assert_eq!(spec["info"]["version"], "v1");
}

// The spec lists the health route with its operation id
#[tokio::test]
async fn spec_lists_the_health_route() {
    let spec = spec().await;
    let operation = &spec["paths"]["/api/v1/health"]["get"];

    assert!(operation.is_object());
    assert_eq!(operation["operationId"], "health");
}

// The spec goes through the gate and redirects without credentials
#[tokio::test]
async fn spec_is_gated() {
    let response = get("/openapi/json", &[]).await;

    assert_eq!(response.status(), StatusCode::FOUND);
}

// GET /openapi serves the Scalar page once the gate is passed
#[tokio::test]
async fn scalar_page_is_served() {
    let response = get("/openapi", &[("cookie", "auth=ok")]).await;

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();

    assert!(html.contains("Gmail data gateway API"));
}
