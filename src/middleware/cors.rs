use axum::http::{HeaderName, HeaderValue, Method, header};
use tower_http::cors::{AllowOrigin, CorsLayer};

// Exact origins allowed to call the API from a browser (SPEC section 4.4)
const FIXED_ORIGINS: [&str; 1] = ["http://localhost:3000"];

// Origin suffixes allowed to call the API from a browser (SPEC section 4.4)
const WILDCARD_SUFFIXES: [&str; 4] = [
    ".trycloudflare.com",
    ".devtunnels.ms",
    ".vercel.app",
    ".dheerajy1dev.dpdns.org",
];

// Checks an Origin header value against the fixed list and the suffix list
fn is_allowed_origin(origin: &HeaderValue) -> bool {
    let Ok(origin) = origin.to_str() else {
        return false;
    };

    FIXED_ORIGINS.contains(&origin)
        || WILDCARD_SUFFIXES
            .iter()
            .any(|suffix| origin.ends_with(suffix))
}

// Builds the CORS layer, requests without an Origin header are untouched
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            is_allowed_origin(origin)
        }))
        .allow_credentials(true)
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            HeaderName::from_static("x-client-id"),
            HeaderName::from_static("x-client-secret"),
        ])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
}
