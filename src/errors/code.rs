use axum::http::StatusCode;

// Every error code the jobs API uses, see SPEC section 11 (D19: only used codes are ported)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotSupported,
    TooManyRequests,
    InternalServerError,
    ServiceUnavailable,
}

impl ErrorCode {
    // Machine readable code sent in the response body
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BadRequest => "BAD_REQUEST",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden => "FORBIDDEN",
            Self::NotFound => "NOT_FOUND",
            Self::MethodNotSupported => "METHOD_NOT_SUPPORTED",
            Self::TooManyRequests => "TOO_MANY_REQUESTS",
            Self::InternalServerError => "INTERNAL_SERVER_ERROR",
            Self::ServiceUnavailable => "SERVICE_UNAVAILABLE",
        }
    }

    // Numeric http status sent in the response body
    pub fn status_code(self) -> u16 {
        match self {
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::MethodNotSupported => 405,
            Self::TooManyRequests => 429,
            Self::InternalServerError => 500,
            Self::ServiceUnavailable => 503,
        }
    }

    // Http status used on the response line
    pub fn status(self) -> StatusCode {
        StatusCode::from_u16(self.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }

    // Generic summary used when no specific catalogue entry is given
    pub fn message(self) -> &'static str {
        match self {
            Self::BadRequest => {
                "The server cannot process the request because the input provided is invalid."
            }
            Self::Unauthorized => {
                "The client request has not been completed because it lacks valid authentication credentials for the requested resource."
            }
            Self::Forbidden => "You do not have permission to access the requested resource.",
            Self::NotFound => "The requested resource could not be found.",
            Self::MethodNotSupported => {
                "The server knows the request method, but the target resource doesn't support this method."
            }
            Self::TooManyRequests => {
                "The rate limit has been exceeded or too many requests are being sent to the server."
            }
            Self::InternalServerError => {
                "An unexpected error occurred while processing your request."
            }
            Self::ServiceUnavailable => "The server is not ready to handle the request.",
        }
    }
}
