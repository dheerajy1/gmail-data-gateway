use std::sync::Once;

use axum::{
    Router,
    body::Body,
    http::{Request, Response},
};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

static INIT: Once = Once::new();

// Sets the env variables once, before the global env is first read
pub fn setup_env() {
    INIT.call_once(|| {
        // Runs before any router is built, so no other thread reads the env yet
        unsafe {
            std::env::set_var("APP_NAME", "jobs-api-gateway");
            std::env::set_var("APP_ENV", "development");
            std::env::set_var("APP_PORT", "5000");
            std::env::set_var("CLIENT_ID", "test-client");
            std::env::set_var("CLIENT_SECRET", "test-secret");
            std::env::set_var("JWT_SECRET", "test-jwt-secret");
            std::env::set_var("AZURE_DB_SERVER", "test-server");
            std::env::set_var("AZURE_DB_DATABASE", "test-db");
            std::env::set_var("AZURE_DB_USER_PUBLIC", "test-public-user");
            std::env::set_var("AZURE_DB_PASSWORD_PUBLIC", "test-public-password");
            std::env::set_var("AZURE_DB_USER_SERVICE", "test-service-user");
            std::env::set_var("AZURE_DB_PASSWORD_SERVICE", "test-service-password");
            std::env::set_var("ONPREM_SERVER_URL", "http://127.0.0.1:9");
            std::env::set_var("KAFKA_API_KEY", "test-kafka-key");
            std::env::set_var("KAFKA_API_KEY_SECRET", "test-kafka-secret");
            std::env::set_var("SOURCE_CLIENT_ID", "test-source");
            std::env::set_var("JOBS_WRITE_TOPIC", "test-write-topic");
            std::env::set_var("JOBS_WRITE_DLQ_TOPIC", "test-write-dlq");
            std::env::set_var("JOBS_READ_TOPIC", "test-read-topic");
            std::env::set_var("JOBS_READ_DLQ_TOPIC", "test-read-dlq");
            std::env::set_var("JOBS_STATUS_TOPIC", "test-status-topic");
            std::env::set_var("ONPREM_DB_SERVER", "test-onprem");
            std::env::set_var("ONPREM_DB_PORT", "5432");
            std::env::set_var("ONPREM_DB_DATABASE", "test-onprem-db");
            std::env::set_var("ONPREM_DB_USER_SERVICE", "test-onprem-user");
            std::env::set_var("ONPREM_DB_PASSWORD_SERVICE", "test-onprem-password");
            std::env::set_var("Google_APPID", "1");
        }
    });
}

// Builds the real application router with the test env
pub fn app() -> Router {
    setup_env();
    jobs_api_gateway::app::router()
}

// Sends a request with the given method through a fresh router
pub async fn send(method: &str, uri: &str) -> Response<Body> {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    app().oneshot(request).await.unwrap()
}

// Reads the whole response body as json
pub async fn body_json(response: Response<Body>) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}
