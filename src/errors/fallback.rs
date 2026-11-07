use super::{api_error::ApiError, catalog::method_not_supported, catalog::not_found};

// Router fallback for paths that match no route
pub async fn not_found() -> ApiError {
    ApiError::from_entry(not_found::RESOURCE_NOT_FOUND)
}

// Router fallback for routes that exist but not for the requested method
pub async fn method_not_allowed() -> ApiError {
    ApiError::from_entry(method_not_supported::METHOD_NOT_ALLOWED)
}
