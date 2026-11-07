use axum::{extract::FromRequestParts, http::request::Parts};
use validator::Validate;

use crate::{
    config::env::env,
    errors::{
        api_error::ApiError,
        catalog::{bad_request, unauthorized},
    },
    schemas::auth::{CLIENT_ID_HEADER, CLIENT_SECRET_HEADER, NoAuthHeaders},
    security::secrets_match,
};

// True when the headers carry the configured client id and secret, compared in constant time (D35)
pub fn credentials_match(headers: &NoAuthHeaders) -> bool {
    let env = env();

    // Both values are always compared, so the time does not show which one was wrong
    let id_matches = secrets_match(&headers.client_id, &env.client_id);
    let secret_matches = secrets_match(&headers.client_secret, &env.client_secret);

    id_matches && secret_matches
}

// Extractor for client-header endpoints, a handler that takes it only runs with valid client credentials
#[derive(Clone)]
pub struct NoAuth(pub NoAuthHeaders);

impl<S> FromRequestParts<S> for NoAuth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let client_id = parts.headers.get(CLIENT_ID_HEADER);
        let client_secret = parts.headers.get(CLIENT_SECRET_HEADER);

        // Missing headers return 401
        let (Some(client_id), Some(client_secret)) = (client_id, client_secret) else {
            return Err(ApiError::from_entry(unauthorized::MISSING_AUTH_HEADERS));
        };

        // Header values that are not readable text return 400
        let (Ok(client_id), Ok(client_secret)) = (client_id.to_str(), client_secret.to_str())
        else {
            return Err(ApiError::from_entry(bad_request::INVALID_HEADER_VALUES));
        };

        let headers = NoAuthHeaders::new(client_id, client_secret);

        // Present but empty or too long returns 400
        if headers.validate().is_err() {
            return Err(ApiError::from_entry(bad_request::INVALID_HEADER_VALUES));
        }

        // Present but not matching the configured credentials returns 400
        if !credentials_match(&headers) {
            return Err(ApiError::from_entry(bad_request::INVALID_HEADER_VALUES));
        }

        Ok(Self(headers))
    }
}
