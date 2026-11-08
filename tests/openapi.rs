mod common;

use axum::{
    body::Body,
    http::{Response, StatusCode},
};
use common::{auth_headers, body_json, body_text, send};
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

// The spec lists the API login route with its operation id
#[tokio::test]
async fn spec_lists_the_login_route() {
    let spec = spec().await;
    let operation = &spec["paths"]["/api/v1/auth/login"]["post"];

    assert!(operation.is_object());
    assert_eq!(operation["operationId"], "auth_login");
}

// The login operation documents its error responses by status
#[tokio::test]
async fn login_documents_error_responses() {
    let spec = spec().await;
    let responses = &spec["paths"]["/api/v1/auth/login"]["post"]["responses"];

    for status in ["200", "400", "401", "500"] {
        assert!(responses[status].is_object(), "missing status {status}");
    }

    let examples = &responses["401"]["content"]["application/json"]["examples"];

    assert!(examples["Invalid credentials"].is_object());
}

// The health operation documents the 503 response
#[tokio::test]
async fn health_documents_the_503_response() {
    let spec = spec().await;
    let responses = &spec["paths"]["/api/v1/health"]["get"]["responses"];

    assert!(responses["503"].is_object());
}

// The error envelope is listed as a schema
#[tokio::test]
async fn spec_lists_the_error_schema() {
    let spec = spec().await;

    assert!(spec["components"]["schemas"]["ErrorResponse"].is_object());
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

    let html = body_text(response).await;

    assert!(html.contains("Gmail data gateway API"));
}

// The spec lists the extract route with its operation id
#[tokio::test]
async fn spec_lists_the_extract_route() {
    let spec = spec().await;
    let operation = &spec["paths"]["/api/v1/admin/extractMails"]["post"];

    assert!(operation.is_object());
    assert_eq!(operation["operationId"], "extract_mails");
}

// The extract operation documents its error responses by status
#[tokio::test]
async fn extract_documents_error_responses() {
    let spec = spec().await;
    let responses = &spec["paths"]["/api/v1/admin/extractMails"]["post"]["responses"];

    for status in ["200", "400", "401", "403", "404", "429", "500"] {
        assert!(responses[status].is_object(), "missing status {status}");
    }

    let examples = &responses["500"]["content"]["application/json"]["examples"];

    assert!(examples["Kafka gateway communication error"].is_object());
}
