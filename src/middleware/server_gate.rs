use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use validator::Validate;

use super::no_auth::credentials_match;
use crate::{
    headers::wants_html,
    schemas::auth::{CLIENT_ID_HEADER, CLIENT_SECRET_HEADER, NoAuthHeaders},
};

// File extensions that skip the gate, same list as the old code (SPEC section 4.2)
const OPEN_EXTENSIONS: [&str; 8] = ["css", "js", "png", "jpg", "jpeg", "svg", "ico", "txt"];

// True when the path ends with one of the open file extensions
fn has_open_extension(path: &str) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, extension)| OPEN_EXTENSIONS.contains(&extension))
}

// True for a non-html request that carries the valid client id and secret headers
fn has_valid_client_headers(headers: &HeaderMap) -> bool {
    if wants_html(headers) {
        return false;
    }

    let (Some(client_id), Some(client_secret)) = (
        headers.get(CLIENT_ID_HEADER),
        headers.get(CLIENT_SECRET_HEADER),
    ) else {
        return false;
    };

    let (Ok(client_id), Ok(client_secret)) = (client_id.to_str(), client_secret.to_str()) else {
        return false;
    };

    let credentials = NoAuthHeaders::new(client_id, client_secret);

    credentials.validate().is_ok() && credentials_match(&credentials)
}

// True when the request carries the logged-in cookie
fn has_auth_cookie(headers: &HeaderMap) -> bool {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .any(|pair| pair.trim() == "auth=ok")
}

// Global gate (SPEC section 4.2), anything not allowed is redirected to /login
pub async fn server_gate(request: Request, next: Next) -> Response {
    let allowed = {
        let path = request.uri().path();
        let headers = request.headers();

        // 1. Files with an extension, 2. server to server headers, 3. the login page, 4. logged in browser
        has_open_extension(path)
            || has_valid_client_headers(headers)
            || path.starts_with("/login")
            || has_auth_cookie(headers)
    };

    if allowed {
        return next.run(request).await;
    }

    // Not logged in, redirect to the login page
    (StatusCode::FOUND, [(header::LOCATION, "/login")]).into_response()
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    // Builds a header map with one Cookie header
    fn cookie(value: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert(header::COOKIE, HeaderValue::from_static(value));

        headers
    }

    // Every listed extension skips the gate
    #[test]
    fn listed_extensions_are_open() {
        for extension in OPEN_EXTENSIONS {
            assert!(has_open_extension(&format!("/file.{extension}")));
        }
    }

    // Other extensions and paths without an extension do not skip the gate
    #[test]
    fn other_paths_are_not_open() {
        assert!(!has_open_extension("/api/v1/health"));
        assert!(!has_open_extension("/openapi/json"));
        assert!(!has_open_extension("/file.php"));
        assert!(!has_open_extension("/archive.tar.gz"));
        assert!(!has_open_extension("/css"));
    }

    // The cookie auth=ok is found alone and next to other cookies
    #[test]
    fn auth_cookie_is_found() {
        assert!(has_auth_cookie(&cookie("auth=ok")));
        assert!(has_auth_cookie(&cookie("theme=dark; auth=ok")));
    }

    // A wrong value, a longer name or no cookie at all is not the logged-in cookie
    #[test]
    fn other_cookies_are_not_the_auth_cookie() {
        assert!(!has_auth_cookie(&cookie("auth=no")));
        assert!(!has_auth_cookie(&cookie("xauth=ok")));
        assert!(!has_auth_cookie(&HeaderMap::new()));
    }
}
