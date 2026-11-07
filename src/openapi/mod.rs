use axum::{Json, Router, routing::get};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::features::v1::health::handler as health;

// Title shown on the docs page and in the spec (SPEC section 4.1)
const TITLE: &str = "Gmail data gateway API";

// The OpenAPI document, every new route is added to paths when it is built
#[derive(OpenApi)]
#[openapi(
    info(title = "Gmail data gateway API", version = "v1"),
    paths(health::health)
)]
pub struct ApiDoc;

// GET /openapi/json returns the raw OpenAPI spec
async fn spec() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

// Routes of the docs, GET /openapi serves the Scalar UI and GET /openapi/json the spec
pub fn router() -> Router {
    Router::new()
        .route("/openapi/json", get(spec))
        .merge(Scalar::with_url("/openapi", ApiDoc::openapi()).title(TITLE))
}
