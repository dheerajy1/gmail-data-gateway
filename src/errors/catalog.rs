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

// 404 entries
pub mod not_found {
    use super::{CatalogEntry, ErrorCode};

    pub const RESOURCE_NOT_FOUND: CatalogEntry = CatalogEntry::new(
        ErrorCode::NotFound,
        "The requested resource does not exist or is no longer available.",
        "The requested resource does not exist",
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
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every entry in the catalogue with the http status it must map to
    const ENTRIES: [(CatalogEntry, u16); 3] = [
        (not_found::RESOURCE_NOT_FOUND, 404),
        (method_not_supported::METHOD_NOT_ALLOWED, 405),
        (internal_server_error::SERVER_ERROR, 500),
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
        ];

        for (code, name, status) in codes {
            assert_eq!(code.as_str(), name);
            assert_eq!(code.status_code(), status);
            assert!(!code.message().is_empty());
        }
    }
}
