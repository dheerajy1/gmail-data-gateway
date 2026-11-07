use std::any::Any;

use axum::response::{IntoResponse, Response};

use super::{api_error::ApiError, catalog::internal_server_error};

// Turns a panic inside a handler into the global 500 error shape
pub fn handle_panic(payload: Box<dyn Any + Send + 'static>) -> Response {
    // Panic text is only logged, never sent to the client
    let detail = if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "unknown panic payload".to_string()
    };

    tracing::error!(target: "Server", "handler panicked: {detail}");

    ApiError::from_entry(internal_server_error::SERVER_ERROR).into_response()
}
