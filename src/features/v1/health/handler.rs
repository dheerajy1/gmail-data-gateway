use axum::{Json, Router, routing::get};

use super::{model::HealthData, service};
use crate::schemas::response::SuccessResponse;

// Routes of the health feature, mounted under /api/v1/health
pub fn router() -> Router {
    Router::new().route("/", get(health))
}

// GET /api/v1/health, reachable through the access gate (SPEC section 4.1)
#[utoipa::path(
    get,
    path = "/api/v1/health",
    operation_id = "health",
    tag = "Health",
    summary = "Health check",
    description = "Returns application health. Requires the client credential headers or a logged in browser.",
    params(
        ("x-client-id" = String, Header, description = "Client id"),
        ("x-client-secret" = String, Header, description = "Client secret"),
    ),
    responses(
        (
            status = 200,
            description = "Response for status 200",
            body = SuccessResponse<HealthData>,
            example = json!({
                "success": true,
                "statusCode": 200,
                "message": "Health check OK",
                "data": {
                    "app": {
                        "status": "UP",
                        "version": "0.1.0",
                        "uptimeMs": 123456,
                        "timestampUtc": "2025-01-01T00:00:00.000Z"
                    }
                }
            })
        )
    )
)]
pub async fn health() -> Json<SuccessResponse<HealthData>> {
    Json(SuccessResponse::ok(
        "Health check OK",
        HealthData {
            app: service::get_app_health(),
        },
    ))
}
