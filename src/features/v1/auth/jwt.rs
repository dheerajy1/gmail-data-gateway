use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, errors::ErrorKind};

use super::model::JwtClaims;
use crate::{
    config::env::env,
    errors::{api_error::ApiError, catalog::unauthorized},
};

// Issuer written into every token (SPEC section 4.3)
pub const ISSUER: &str = "jobs-api-gateway";

// Verifies the signature, issuer and expiry of a token and returns its claims (D36, D38)
pub fn verify_token(token: &str) -> Result<JwtClaims, ApiError> {
    let mut validation = Validation::new(Algorithm::HS256);

    validation.set_issuer(&[ISSUER]);

    // No grace period after expiry
    validation.leeway = 0;

    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(env().jwt_secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|error| match error.kind() {
        ErrorKind::ExpiredSignature => ApiError::from_entry(unauthorized::EXPIRED_TOKEN),
        _ => ApiError::from_entry(unauthorized::INVALID_TOKEN),
    })
}
