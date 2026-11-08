use super::{
    model::{ExtractMailsRequest, ExtractType},
    query::{BuiltQuery, build_query},
};
use crate::{db::applied_jobs::get_last_applied_unix, errors::api_error::ApiError};

// Validates a request and builds its Gmail query, Sync reads the last stored date first
pub async fn prepare_query(request: &ExtractMailsRequest) -> Result<BuiltQuery, ApiError> {
    request.validate()?;

    let last_stored_unix = match request.extract_type {
        ExtractType::Sync => Some(get_last_applied_unix().await?),
        ExtractType::Bulk | ExtractType::Increment => None,
    };

    build_query(request, last_stored_unix)
}
