use std::fmt;

use axum::{
    Json,
    response::{IntoResponse, Response},
};

use super::{catalog::CatalogEntry, code::ErrorCode};
use crate::schemas::response::ErrorResponse;

// The one error type handlers return, rendered as the global error shape
#[derive(Debug, Clone)]
pub struct ApiError {
    code: ErrorCode,
    message: String,
    error: String,
}

impl ApiError {
    // Builds the error from a catalogue entry, the normal way to return an error
    pub fn from_entry(entry: CatalogEntry) -> Self {
        Self {
            code: entry.code,
            message: entry.message.to_string(),
            error: entry.error.to_string(),
        }
    }

    // Builds the error from a code with a custom error text, message comes from the code
    pub fn new(code: ErrorCode, error: impl Into<String>) -> Self {
        Self {
            code,
            message: code.message().to_string(),
            error: error.into(),
        }
    }

    pub fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn error(&self) -> &str {
        &self.error
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.error)
    }
}

impl std::error::Error for ApiError {}

// Lets a catalogue entry be turned into an error with `.into()` or `?`
impl From<CatalogEntry> for ApiError {
    fn from(entry: CatalogEntry) -> Self {
        Self::from_entry(entry)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            success: false,
            status_code: self.code.status_code(),
            code: self.code.as_str(),
            message: self.message,
            error: self.error,
        };

        (self.code.status(), Json(body)).into_response()
    }
}
