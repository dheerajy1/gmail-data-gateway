use subtle::ConstantTimeEq;

use crate::errors::{api_error::ApiError, catalog::internal_server_error};

// Valid bcrypt hash (cost 12) of a random password nobody knows, checked when a user does not exist (D42)
const DUMMY_HASH: &str = "$2b$12$fNP4bEScoPqLCQM3fzs3oOmKKjf8GvozM0wL9l3CJO5VsHl4loMyi";

// Compares two secrets or tokens without stopping at the first different byte (D35, D39)
pub fn secrets_match(left: &str, right: &str) -> bool {
    bool::from(left.as_bytes().ct_eq(right.as_bytes()))
}

// Checks a password against a bcrypt hash on a blocking thread, bcrypt is slow on purpose
// A missing hash (unknown or inactive user) is checked against a dummy hash and never matches, so the response time does not reveal unknown users (D42)
pub async fn verify_password(password: &str, hash: Option<&str>) -> Result<bool, ApiError> {
    let password = password.to_string();

    let (hash, user_exists) = match hash {
        Some(hash) => (hash.to_string(), true),
        None => (DUMMY_HASH.to_string(), false),
    };

    match tokio::task::spawn_blocking(move || bcrypt::verify(password, &hash)).await {
        Ok(Ok(is_valid)) => Ok(is_valid && user_exists),
        // A stored hash that is not valid bcrypt is a server problem, not a wrong password
        Ok(Err(error)) => {
            tracing::error!(target: "Auth", "password verification failed: {error}");
            Err(ApiError::from_entry(internal_server_error::SERVER_ERROR))
        }
        Err(error) => {
            tracing::error!(target: "Auth", "password verification task failed: {error}");
            Err(ApiError::from_entry(internal_server_error::SERVER_ERROR))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Lowest bcrypt cost, keeps the tests fast
    const TEST_COST: u32 = 4;

    // Equal values match
    #[test]
    fn equal_values_match() {
        assert!(secrets_match("test-secret", "test-secret"));
    }

    // Two empty values match
    #[test]
    fn empty_values_match() {
        assert!(secrets_match("", ""));
    }

    // Values that differ in one character do not match
    #[test]
    fn different_values_do_not_match() {
        assert!(!secrets_match("test-secret", "test-secreT"));
    }

    // Values of different length do not match, even when one is a prefix of the other
    #[test]
    fn different_length_values_do_not_match() {
        assert!(!secrets_match("test-secret", "test-secret-longer"));
        assert!(!secrets_match("test-secret", ""));
    }

    // The right password matches its hash
    #[tokio::test]
    async fn right_password_matches() {
        let hash = bcrypt::hash("secret1", TEST_COST).unwrap();

        assert!(verify_password("secret1", Some(&hash)).await.unwrap());
    }

    // A wrong password does not match
    #[tokio::test]
    async fn wrong_password_does_not_match() {
        let hash = bcrypt::hash("secret1", TEST_COST).unwrap();

        assert!(!verify_password("secret2", Some(&hash)).await.unwrap());
    }

    // A missing user never matches, and the dummy hash is a valid bcrypt hash
    #[tokio::test]
    async fn missing_user_never_matches() {
        assert!(!verify_password("secret1", None).await.unwrap());
    }

    // A stored hash that is not bcrypt is a server error
    #[tokio::test]
    async fn invalid_stored_hash_is_a_server_error() {
        assert!(
            verify_password("secret1", Some("not-a-hash"))
                .await
                .is_err()
        );
    }
}
