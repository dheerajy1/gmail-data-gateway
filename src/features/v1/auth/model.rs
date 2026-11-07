use serde::{Deserialize, Serialize};

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
