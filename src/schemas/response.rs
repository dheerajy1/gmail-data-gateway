use serde::Serialize;
use utoipa::ToSchema;

// Success envelope shared by every API endpoint
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SuccessResponse<T: Serialize> {
    pub success: bool,
    pub status_code: u16,
    pub message: String,
    pub data: T,
}

impl<T: Serialize> SuccessResponse<T> {
    // Builds a 200 OK envelope around the given data
    pub fn ok(message: &str, data: T) -> Self {
        Self {
            success: true,
            status_code: 200,
            message: message.to_string(),
            data,
        }
    }

    // Builds a 202 Accepted envelope around the given data
    pub fn accepted(message: &str, data: T) -> Self {
        Self {
            success: true,
            status_code: 202,
            message: message.to_string(),
            data,
        }
    }
}

// Error envelope shared by every API error
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub success: bool,
    pub status_code: u16,
    pub code: &'static str,
    pub message: String,
    pub error: String,
}
