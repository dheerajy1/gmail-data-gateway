use jobs_api_gateway::{app, config::env, features::v1::health::service, logging};
use tower::ServiceBuilder;
use vercel_runtime::Error;
use vercel_runtime::axum::VercelLayer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Installs the logger first so every later step can log
    logging::init();

    // Loads and validates the env at boot, the process exits if it is invalid
    env::init();

    // Records the process start time so uptime counts from boot
    service::mark_started();

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(app::router());

    vercel_runtime::run(app).await
}
