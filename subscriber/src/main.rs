use std::sync::Arc;

use jobs_api_gateway::{config::env, logging};
use subscriber::{
    gateway::{self, Backoff, GatewayConfig, GatewayEvent},
    topics::Topics,
};
use tokio::signal::unix::{SignalKind, signal};

// Log target of the subscriber process
const LOG_TARGET: &str = "JobsSub";

// Classifies an event by its topic, the stage handlers arrive in P8 and until then nothing is acknowledged
async fn on_event(topics: Arc<Topics>, event: GatewayEvent) {
    match topics.route(&event.topic) {
        Some(route) => tracing::info!(
            target: LOG_TARGET,
            "event received route={route:?} topic={} (handlers arrive in P8, not acknowledged)",
            event.topic
        ),
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
