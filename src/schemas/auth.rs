use serde::Deserialize;
use validator::Validate;

// Header that carries the client id
pub const CLIENT_ID_HEADER: &str = "x-client-id";

// Header that carries the client secret
pub const CLIENT_SECRET_HEADER: &str = "x-client-secret";

// Required client credential headers for API endpoints (SPEC section 4.3)
#[derive(Clone, Validate)]
pub struct NoAuthHeaders {
    // x-client-id used for authentication
    #[validate(length(
        min = 1,
        max = 30,
        message = "x-client-id must be between 1 and 30 characters."
    ))]
    pub client_id: String,

    // x-client-secret used for authentication
    #[validate(length(
        min = 1,
        max = 50,
        message = "x-client-secret must be between 1 and 50 characters."
    ))]
    pub client_secret: String,
}

impl NoAuthHeaders {
    // Builds the headers struct, trimming whitespace like the Elysia schema did
    pub fn new(client_id: &str, client_secret: &str) -> Self {
        Self {
            client_id: client_id.trim().to_string(),
            client_secret: client_secret.trim().to_string(),
        }
    }
}

// Form fields posted by the portal login page
#[derive(Deserialize, Validate)]
pub struct LoginForm {
    // Portal user name
    pub username: String,

    // Portal password, same minimum length as Elysia
    #[validate(length(min = 6, message = "Password must be at least 6 characters."))]
    pub password: String,
}
