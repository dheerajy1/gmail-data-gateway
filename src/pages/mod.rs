pub mod about;
pub mod healthz;
pub mod home;
pub mod login;
pub mod render;

use axum::Router;

// All HTML page routes, mounted at the root
pub fn router() -> Router {
    Router::new()
        .merge(home::router())
        .merge(about::router())
        .merge(healthz::router())
        .merge(login::router())
}
