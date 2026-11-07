use axum::{Json, Router, routing::get};

use super::{
    model::{DbStatus, HealthData},
    service,
};
use crate::{
    errors::{api_error::ApiError, catalog::service_unavailable},
    schemas::response::SuccessResponse,
};

// Routes of the health feature, mounted under /api/v1/health
pub fn router() -> Router {
    Router::new().route("/", get(health))
}

// GET /api/v1/health, reachable through the access gate (SPEC section 4.1), the error responses are added in openapi/errors.rs under the operation id health
#[utoipa::path(
    get,
    path = "/api/v1/health",
    operation_id = "health",
    tag = "Health",
    summary = "Health check",
    description = "Returns application and database health. Requires the client credential headers or a logged in browser. Returns 503 while the database is down.",
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
                    },
                    "db": {
                        "status": "UP",
                        "database": "CONNECTED",
                        "latencyMs": 12
                    }
                }
            })
        )
    )
)]
pub async fn health() -> Result<Json<SuccessResponse<HealthData>>, ApiError> {
    let app = service::get_app_health();
    let db = service::get_db_health().await;

    // A database that is down fails the check with 503 (D43)
    if db.status == DbStatus::Down {
        return Err(ApiError::from_entry(service_unavailable::DB_PROBE_FAILED));
    }

    Ok(Json(SuccessResponse::ok(
        "Health check OK",
        HealthData { app, db },
    )))
}
