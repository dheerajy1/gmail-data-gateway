use axum::{Json, Router, extract::rejection::JsonRejection, routing::post};
use validator::Validate;

use super::{
    model::{LoginData, LoginRequest},
    service,
};
use crate::{
    errors::{api_error::ApiError, catalog::bad_request},
    middleware::no_auth::NoAuth,
    schemas::response::SuccessResponse,
};

// Routes of the auth feature, mounted under /api/v1/auth
pub fn router() -> Router {
    Router::new().route("/login", post(login))
}

// POST /api/v1/auth/login, the error responses are added in openapi/errors.rs under the operation id auth_login
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    operation_id = "auth_login",
    tag = "Auth",
    summary = "API user login",
    description = "Authenticate a user and return access token",
    params(
        ("x-client-id" = String, Header, description = "Client id"),
        ("x-client-secret" = String, Header, description = "Client secret"),
    ),
    request_body = LoginRequest,
    responses(
        (
            status = 200,
            description = "Response for status 200",
            body = SuccessResponse<LoginData>,
            example = json!({
                "success": true,
                "statusCode": 200,
                "message": "Login successful",
                "data": {
                    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
                    "expiresAt": "2025-01-01T00:00:00.000Z"
                }
            })
        )
    )
)]
pub async fn login(
    _auth: NoAuth,
    body: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<Json<SuccessResponse<LoginData>>, ApiError> {
    // A body that is not valid json returns 400
    let Ok(Json(request)) = body else {
        return Err(ApiError::from_entry(bad_request::INVALID_INPUT));
    };

    // An empty username or a short password returns 400
    if request.validate().is_err() {
        return Err(ApiError::from_entry(bad_request::INVALID_INPUT));
    }

    let data = service::login(request).await?;

    Ok(Json(SuccessResponse::ok("Login successful", data)))
}
