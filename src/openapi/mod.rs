pub mod errors;

use axum::{Json, Router, routing::get};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use self::errors::ErrorExamples;

use crate::{
    features::v1::{auth::handler as auth, health::handler as health},
    schemas::response::ErrorResponse,
};

// Title shown on the docs page and in the spec (SPEC section 4.1)
const TITLE: &str = "Gmail data gateway API";

// The OpenAPI document, routes are listed in paths and error examples are added by ErrorExamples
#[derive(OpenApi)]
#[openapi(
    info(title = "Gmail data gateway API", version = "v1"),
    paths(auth::login, health::health),
    components(schemas(ErrorResponse)),
    modifiers(&ErrorExamples)
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
