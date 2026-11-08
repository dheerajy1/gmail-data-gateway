pub mod auth;
pub mod extract_mails;
pub mod health;

use axum::Router;

// All v1 routes, nested under /api/v1
pub fn router() -> Router {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .nest("/auth", auth::handler::router())
            .nest("/health", health::handler::router()),
    )
}
