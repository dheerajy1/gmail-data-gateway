use axum::{Router, http::header, response::IntoResponse, routing::get};
use tower_http::catch_panic::CatchPanicLayer;

use crate::{
    errors::{fallback, panic::handle_panic},
    features::v1,
};

// Serves the precompiled Tailwind stylesheet embedded in the binary
async fn global_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../public/global.css"),
    )
}

// Serves the logo image embedded in the binary
async fn logo() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/png")],
        include_bytes!("../public/logo.png").as_slice(),
    )
}

// Serves the favicon embedded in the binary
async fn favicon() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/x-icon")],
        include_bytes!("../public/favicon.ico").as_slice(),
    )
}

// Full application router, shared by main and the tests
pub fn router() -> Router {
    Router::new()
        .route("/global.css", get(global_css))
        .route("/logo.png", get(logo))
        .route("/favicon.ico", get(favicon))
        .merge(v1::router())
        // Unknown paths return the global 404 error shape
        .fallback(fallback::not_found)
        // Known paths with a wrong method return the global 405 error shape
        .method_not_allowed_fallback(fallback::method_not_allowed)
        // A panic in any handler becomes the global 500 error shape
        .layer(CatchPanicLayer::custom(handle_panic))
}
