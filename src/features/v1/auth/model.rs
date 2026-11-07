use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

// Body of POST /api/v1/auth/login
#[derive(Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    // Api user name
    #[validate(length(min = 1, message = "username is required"))]
    pub username: String,

    // Api user password, same minimum length as Elysia
    #[validate(length(min = 6, message = "Password must be at least 6 characters."))]
    pub password: String,
}

// Data returned by a successful login
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginData {
    // Signed bearer token
    pub token: String,

    // Token expiry as an ISO 8601 UTC timestamp
    pub expires_at: String,
}

// Claims inside the signed token: subject, role, issuer, issue time and expiry (SPEC section 4.3)
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    // Api user name, empty when the token has none
    #[serde(default)]
    pub sub: String,

    // Api user role, empty when the token has none
    #[serde(default)]
    pub role: String,

    // Issuer of the token
    pub iss: String,

    // Issue time as seconds since the unix epoch, zero when the token has none
    #[serde(default)]
    pub iat: u64,

    // Expiry as seconds since the unix epoch
    pub exp: u64,
}
