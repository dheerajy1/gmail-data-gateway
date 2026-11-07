use axum::http::{HeaderMap, header};

// True when the request Accept header asks for html, as a browser does (D37)
pub fn wants_html(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|accept| accept.contains("text/html"))
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    // Builds a header map with one Accept header
    fn accept(value: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();

        headers.insert(header::ACCEPT, HeaderValue::from_static(value));

        headers
    }

    // A browser Accept header asks for html
    #[test]
    fn browser_accept_wants_html() {
        assert!(wants_html(&accept(
            "text/html,application/xhtml+xml,*/*;q=0.8"
        )));
    }

    // A json Accept header does not ask for html
    #[test]
    fn json_accept_does_not_want_html() {
        assert!(!wants_html(&accept("application/json")));
    }

    // A request without an Accept header does not ask for html
    #[test]
    fn missing_accept_does_not_want_html() {
        assert!(!wants_html(&HeaderMap::new()));
    }
}
