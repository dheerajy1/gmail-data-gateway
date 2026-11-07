use axum::{Router, response::Html, routing::get};

use super::render::render_with_props;
use crate::components::about::{About, AboutProps};

// Text shown on the About page
const ABOUT_DESCRIPTION: &str = "Gmail data gateway API is a backend built with Rust, Axum and Dioxus server-side rendering. It extracts Gmail messages and publishes them to Kafka, with every page rendered on the server and styled with Tailwind.";

// Routes of the About page
pub fn router() -> Router {
    Router::new().route("/about", get(about))
}

// GET /about
async fn about() -> Html<String> {
    render_with_props(
        About,
        AboutProps {
            title: "About | Gmail data gateway API".to_string(),
            heading: "About".to_string(),
            description: ABOUT_DESCRIPTION.to_string(),
        },
    )
}
