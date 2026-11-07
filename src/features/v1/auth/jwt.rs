use chrono::{DateTime, SecondsFormat, Utc};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::ErrorKind,
};

use super::model::{JwtClaims, LoginData};
use crate::{
    config::env::env,
    errors::{
        api_error::ApiError,
        catalog::{internal_server_error, unauthorized},
    },
};

// Issuer written into every token (SPEC section 4.3)
pub const ISSUER: &str = "jobs-api-gateway";

// Token lifetime of one day (SPEC section 4.3)
const TOKEN_LIFETIME_SECONDS: u64 = 86_400;

// Signs a token for an api user and returns it with its expiry time
pub fn sign_token(username: &str, role: &str) -> Result<LoginData, ApiError> {
    let issued_at = Utc::now().timestamp().unsigned_abs();
    let exp = issued_at + TOKEN_LIFETIME_SECONDS;

    let claims = JwtClaims {
        sub: username.to_string(),
        role: role.to_string(),
        iss: ISSUER.to_string(),
        iat: issued_at,
        exp,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(env().jwt_secret.as_bytes()),
    )
    .map_err(|_| ApiError::from_entry(internal_server_error::TOKEN_SIGNING_FAILED))?;

    let expires_at = i64::try_from(exp)
        .ok()
        .and_then(|exp| DateTime::from_timestamp(exp, 0))
        .ok_or_else(|| ApiError::from_entry(internal_server_error::TOKEN_SIGNING_FAILED))?
        .to_rfc3339_opts(SecondsFormat::Millis, true);

    Ok(LoginData { token, expires_at })
}

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
