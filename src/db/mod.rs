use std::fmt::Display;

use crate::errors::{api_error::ApiError, catalog::internal_server_error};

pub mod api_users;
pub mod applied_jobs;
pub mod azure;
pub mod google_oauth;
pub mod portal_users;

// Logs the real database error and returns the generic database error to the caller
pub fn db_error(error: impl Display) -> ApiError {
    tracing::error!(target: "Db", "query failed: {error}");

    ApiError::from_entry(internal_server_error::DATABASE_ERROR)
}
