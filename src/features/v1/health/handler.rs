use axum::{Json, Router, routing::get};

use super::{model::HealthData, service};
use crate::schemas::response::SuccessResponse;

// Routes of the health feature, mounted under /api/v1/health
pub fn router() -> Router {
    Router::new().route("/", get(health))
}

// GET /api/v1/health
pub async fn health() -> Json<SuccessResponse<HealthData>> {
    Json(SuccessResponse::ok(
        "Health check OK",
        HealthData {
            app: service::get_app_health(),
        },
    ))
}
