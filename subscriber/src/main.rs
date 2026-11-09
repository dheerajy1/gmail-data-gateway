use std::sync::Arc;

use jobs_api_gateway::{config::env, logging};
use subscriber::{
    gateway::{self, Backoff, GatewayConfig, GatewayEvent},
    handlers::{
        read_model::handle_read_model_event, status::handle_status_event,
        write_model::handle_write_model_event,
    },
    topics::{Route, Topics},
};
use tokio::signal::unix::{SignalKind, signal};

// Log target of the subscriber process
const LOG_TARGET: &str = "JobsSub";

// Sends an event to the handler of its topic, an unknown topic is ignored and not acknowledged
async fn on_event(topics: Arc<Topics>, event: GatewayEvent) {
    match topics.route(&event.topic) {
        Some(Route::Write) => handle_write_model_event(event).await,
        Some(Route::Read) => handle_read_model_event(event).await,
        Some(Route::Status) => handle_status_event(event).await,
        None => tracing::warn!(
            target: LOG_TARGET,
            "ignored event of unknown topic={}",
            event.topic
        ),
    }
}

#[tokio::main]
async fn main() {
    logging::init();
    env::init();

    let topics = Arc::new(Topics::from_env());

    tracing::info!(
        target: LOG_TARGET,
        "start topics={}",
        topics.subscribed().join(",")
    );

    let mut terminate = match signal(SignalKind::terminate()) {
        Ok(terminate) => terminate,
        Err(error) => {
            tracing::error!(target: LOG_TARGET, "cannot listen for SIGTERM: {error}");

            std::process::exit(1);
        }
    };

    let handler = {
        let topics = Arc::clone(&topics);

        move |event: GatewayEvent| on_event(Arc::clone(&topics), event)
    };

    tokio::select! {
        _ = gateway::run(
            GatewayConfig::from_env(),
            topics.subscribed(),
            handler,
            Backoff::STANDARD,
        ) => {}
        _ = tokio::signal::ctrl_c() => {
            tracing::info!(target: LOG_TARGET, "shutdown on SIGINT");
        }
        _ = terminate.recv() => {
            tracing::info!(target: LOG_TARGET, "shutdown on SIGTERM");
        }
    }
}
