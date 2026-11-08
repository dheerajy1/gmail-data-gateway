use super::code::ErrorCode;

// One catalogue entry, a fixed code with its user message and short error text
#[derive(Debug, Clone, Copy)]
pub struct CatalogEntry {
    pub code: ErrorCode,
    pub message: &'static str,
    pub error: &'static str,
}

impl CatalogEntry {
    const fn new(code: ErrorCode, message: &'static str, error: &'static str) -> Self {
        Self {
            code,
            message,
            error,
        }
    }
}

// 401 entries
pub mod unauthorized {
    use super::{CatalogEntry, ErrorCode};

    pub const MISSING_AUTH_HEADERS: CatalogEntry = CatalogEntry::new(
        ErrorCode::Unauthorized,
        "Authentication information is required to continue.",
        "Missing required authentication headers",
    );

    pub const MISSING_AUTHORIZATION_TOKEN: CatalogEntry = CatalogEntry::new(
        ErrorCode::Unauthorized,
        "You must be logged in to access this resource.",
        "Missing required Authorization token",
    );

    pub const EXPIRED_TOKEN: CatalogEntry = CatalogEntry::new(
        ErrorCode::Unauthorized,
        "Your session has expired. Please log in again.",
        "Expired token",
    );

    pub const INVALID_TOKEN: CatalogEntry = CatalogEntry::new(
        ErrorCode::Unauthorized,
        "Your session is no longer valid. Please log in again.",
        "Invalid token",
    );

    pub const MISSING_SUB_OR_ROLE: CatalogEntry = CatalogEntry::new(
        ErrorCode::Unauthorized,
        "JWT is missing required subject (sub) or role claim",
        "missing_sub_or_role",
    );

    pub const INVALID_CREDENTIALS: CatalogEntry = CatalogEntry::new(
        ErrorCode::Unauthorized,
        "The username or password you entered is incorrect.",
        "Invalid credentials",
    );

    pub const GMAIL_SESSION_EXPIRED: CatalogEntry = CatalogEntry::new(
        ErrorCode::Unauthorized,
        "Your Gmail session has expired. Please reconnect your Gmail account.",
        "Gmail session expired",
    );

    pub const NO_ACTIVE_CREDENTIALS: CatalogEntry = CatalogEntry::new(
        ErrorCode::Unauthorized,
        "No active Google credentials are available for this application.",
        "No active credentials found",
    );
}

// 400 entries
pub mod bad_request {
    use super::{CatalogEntry, ErrorCode};

    pub const INVALID_HEADER_VALUES: CatalogEntry = CatalogEntry::new(
        ErrorCode::BadRequest,
        "Some request information is invalid or malformed.",
        "Invalid authentication header values",
    );

    pub const INVALID_INPUT: CatalogEntry = CatalogEntry::new(
        ErrorCode::BadRequest,
        "One or more input fields contain invalid values.",
        "Invalid input data",
    );
}

// 403 entries
pub mod forbidden {
    use super::{CatalogEntry, ErrorCode};

    pub const ACCESS_DENIED: CatalogEntry = CatalogEntry::new(
        ErrorCode::Forbidden,
        "You are not allowed to perform this action.",
        "You do not have permission to perform this action",
    );

    pub const GMAIL_ACCESS_DENIED: CatalogEntry = CatalogEntry::new(
        ErrorCode::Forbidden,
        "Access to the Gmail account was denied.",
        "Gmail access denied",
    );
}

// 404 entries
pub mod not_found {
    use super::{CatalogEntry, ErrorCode};

    pub const RESOURCE_NOT_FOUND: CatalogEntry = CatalogEntry::new(
        ErrorCode::NotFound,
        "The requested resource does not exist or is no longer available.",
        "The requested resource does not exist",
    );

    pub const NO_EMAIL_DATA: CatalogEntry = CatalogEntry::new(
        ErrorCode::NotFound,
        "No stored email data was found to continue from.",
        "No email data found",
    );

    pub const CLIENT_ID_NOT_FOUND: CatalogEntry = CatalogEntry::new(
        ErrorCode::NotFound,
        "The Google OAuth client data was not found.",
        "Google OAuth client not found",
    );
}

// 405 entries
pub mod method_not_supported {
    use super::{CatalogEntry, ErrorCode};

    pub const METHOD_NOT_ALLOWED: CatalogEntry = CatalogEntry::new(
        ErrorCode::MethodNotSupported,
        "The HTTP method used is not supported for this endpoint.",
        "Method not allowed",
    );
}

// 500 entries
pub mod internal_server_error {
    use super::{CatalogEntry, ErrorCode};

    pub const SERVER_ERROR: CatalogEntry = CatalogEntry::new(
        ErrorCode::InternalServerError,
        "Something went wrong. Please try again later.",
        "Something went wrong on the server",
    );

    pub const TOKEN_SIGNING_FAILED: CatalogEntry = CatalogEntry::new(
        ErrorCode::InternalServerError,
        "Failed to generate authentication token due to an internal error.",
        "Token signing failed",
    );

    pub const DATABASE_ERROR: CatalogEntry = CatalogEntry::new(
        ErrorCode::InternalServerError,
        "A database error occurred while processing the request.",
        "Database error",
    );

    pub const KAFKA_GATEWAY_FAILED: CatalogEntry = CatalogEntry::new(
        ErrorCode::InternalServerError,
        "Failed to communicate with the Kafka gateway service.",
        "Kafka gateway communication error",
    );

    pub const EXTRACT_FAILED: CatalogEntry = CatalogEntry::new(
        ErrorCode::InternalServerError,
        "Failed to extract emails.",
        "Email extraction failed",
    );
}

// 429 entries
pub mod too_many_requests {
    use super::{CatalogEntry, ErrorCode};

    pub const GMAIL_RATE_LIMIT: CatalogEntry = CatalogEntry::new(
        ErrorCode::TooManyRequests,
        "Gmail is limiting requests right now. Please try again later.",
        "Gmail rate limit exceeded",
    );
}

// 503 entries
pub mod service_unavailable {
    use super::{CatalogEntry, ErrorCode};

    pub const DB_PROBE_FAILED: CatalogEntry = CatalogEntry::new(
        ErrorCode::ServiceUnavailable,
        "Database connection validation probe failed",
        "Database is disconnected or unreachable",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every entry in the catalogue with the http status it must map to
    const ENTRIES: [(CatalogEntry, u16); 23] = [
        (unauthorized::MISSING_AUTH_HEADERS, 401),
        (unauthorized::MISSING_AUTHORIZATION_TOKEN, 401),
        (unauthorized::EXPIRED_TOKEN, 401),
        (unauthorized::INVALID_TOKEN, 401),
        (unauthorized::MISSING_SUB_OR_ROLE, 401),
        (unauthorized::INVALID_CREDENTIALS, 401),
        (unauthorized::GMAIL_SESSION_EXPIRED, 401),
        (unauthorized::NO_ACTIVE_CREDENTIALS, 401),
        (bad_request::INVALID_HEADER_VALUES, 400),
        (bad_request::INVALID_INPUT, 400),
        (forbidden::ACCESS_DENIED, 403),
        (forbidden::GMAIL_ACCESS_DENIED, 403),
        (not_found::RESOURCE_NOT_FOUND, 404),
        (not_found::NO_EMAIL_DATA, 404),
        (not_found::CLIENT_ID_NOT_FOUND, 404),
        (too_many_requests::GMAIL_RATE_LIMIT, 429),
        (method_not_supported::METHOD_NOT_ALLOWED, 405),
        (internal_server_error::SERVER_ERROR, 500),
        (internal_server_error::TOKEN_SIGNING_FAILED, 500),
        (internal_server_error::DATABASE_ERROR, 500),
        (internal_server_error::KAFKA_GATEWAY_FAILED, 500),
        (internal_server_error::EXTRACT_FAILED, 500),
        (service_unavailable::DB_PROBE_FAILED, 503),
    ];

    // Each entry maps to the stated http status and has a message and error text
    #[test]
    fn every_entry_maps_to_its_status() {
        for (entry, expected) in ENTRIES {
            assert_eq!(entry.code.status_code(), expected);
            assert_eq!(entry.code.status().as_u16(), expected);
            assert!(!entry.message.is_empty());
            assert!(!entry.error.is_empty());
        }
    }

    // Each error code keeps its machine readable name and status together
    #[test]
    fn every_code_has_name_and_status() {
        let codes = [
            (ErrorCode::BadRequest, "BAD_REQUEST", 400),
            (ErrorCode::Unauthorized, "UNAUTHORIZED", 401),
            (ErrorCode::Forbidden, "FORBIDDEN", 403),
            (ErrorCode::NotFound, "NOT_FOUND", 404),
            (ErrorCode::MethodNotSupported, "METHOD_NOT_SUPPORTED", 405),
            (ErrorCode::TooManyRequests, "TOO_MANY_REQUESTS", 429),
            (ErrorCode::InternalServerError, "INTERNAL_SERVER_ERROR", 500),
            (ErrorCode::ServiceUnavailable, "SERVICE_UNAVAILABLE", 503),
        ];

        for (code, name, status) in codes {
            assert_eq!(code.as_str(), name);
            assert_eq!(code.status_code(), status);
            assert!(!code.message().is_empty());
        }
    }
}
