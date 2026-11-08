use axum::{Json, Router, extract::rejection::JsonRejection, routing::post};

use super::{
    model::{ExtractMailsData, ExtractMailsRequest},
    service,
};
use crate::{
    errors::{api_error::ApiError, catalog::bad_request},
    middleware::auth::Auth,
    schemas::response::SuccessResponse,
};

// Routes of the extract feature, mounted under /api/v1/admin
pub fn router() -> Router {
    Router::new().route("/extractMails", post(extract_mails))
}

// POST /api/v1/admin/extractMails, needs the client headers and a bearer token (SPEC section 5), the error responses are added in openapi/errors.rs under the operation id extract_mails
#[utoipa::path(
    post,
    path = "/api/v1/admin/extractMails",
    operation_id = "extract_mails",
    tag = "Admin",
    summary = "Extract mails and publish them",
    description = "Reads mails from Gmail and publishes them as write-model events of at most 100 records each, oldest first. Bulk reads everything that matches the query, Increment reads between the given dates, Sync reads from the last stored date. Azure and PostgreSQL processing is asynchronous.",
    params(
        ("x-client-id" = String, Header, description = "Client id"),
        ("x-client-secret" = String, Header, description = "Client secret"),
        ("authorization" = String, Header, description = "Bearer token from the login endpoint"),
    ),
    request_body = ExtractMailsRequest,
    responses(
        (
            status = 200,
            description = "Response for status 200",
            body = SuccessResponse<ExtractMailsData>,
            example = json!({
                "success": true,
                "statusCode": 200,
                "message": "Bulk accepted. Write-model events published: 3. Azure and PostgreSQL processing is asynchronous.",
                "data": {
                    "correlationIds": [
                        "0190a3c0-7b1e-7000-8000-000000000001",
                        "0190a3c0-7b1e-7000-8000-000000000002",
                        "0190a3c0-7b1e-7000-8000-000000000003"
                    ],
                    "recordCount": 250,
                    "chunkCount": 3,
                    "status": "ACCEPTED"
                }
            })
        )
    )
)]
pub async fn extract_mails(
    _auth: Auth,
    body: Result<Json<ExtractMailsRequest>, JsonRejection>,
) -> Result<Json<SuccessResponse<ExtractMailsData>>, ApiError> {
    // A body that is not valid json returns 400
    let Ok(Json(request)) = body else {
        return Err(ApiError::from_entry(bad_request::INVALID_INPUT));
    };

    let outcome = service::extract_mails(&request).await?;

    Ok(Json(SuccessResponse::ok(&outcome.message, outcome.data)))
}
