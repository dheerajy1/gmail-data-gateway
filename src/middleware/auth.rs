use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};

use super::no_auth::NoAuth;
use crate::{
    errors::{api_error::ApiError, catalog::unauthorized},
    features::v1::auth::jwt::verify_token,
};

// Role stored in the api user table and in the token
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiRole {
    Admin,
    User,
}

impl ApiRole {
    // Reads a role from the token claim, None for anything but admin or user
    fn parse(role: &str) -> Option<Self> {
        match role {
            "admin" => Some(Self::Admin),
            "user" => Some(Self::User),
            _ => None,
        }
    }
}

// Extractor for protected endpoints, a handler that takes it only runs with valid client credentials and a valid bearer token
#[derive(Debug, Clone)]
pub struct Auth {
    // Api user name from the token subject
    pub username: String,

    // Api user role from the token
    pub role: ApiRole,
}

impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Client id and secret are checked first, exactly like the other API endpoints
        NoAuth::from_request_parts(parts, state).await?;

        // A missing, non-bearer or empty Authorization header returns 401
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| ApiError::from_entry(unauthorized::MISSING_AUTHORIZATION_TOKEN))?;

        // A bad signature, wrong issuer or expired token returns 401
        let claims = verify_token(token)?;

        // A token without a subject, or with an unknown role, returns 401
        let Some(role) = ApiRole::parse(&claims.role) else {
            return Err(ApiError::from_entry(unauthorized::MISSING_SUB_OR_ROLE));
        };

        if claims.sub.is_empty() {
            return Err(ApiError::from_entry(unauthorized::MISSING_SUB_OR_ROLE));
        }

        Ok(Self {
            username: claims.sub,
            role,
        })
    }
}
