use jobs_api_gateway::config::env::env;
use serde_json::Value;

use crate::{
    failure::{FailureContext, publish_retry_or_dlq},
    gateway::GatewayEvent,
    status::{PipelineStage, PipelineStatus, publish_status, try_publish_failed_status},
    transport::Transport,
};

pub mod read_model;
pub mod status;
pub mod write_model;

// Log target of every handler log line
const LOG_TARGET: &str = "JobsSub";

// Publishes one pipeline status with details to the status topic and waits for the gateway
pub(crate) async fn report(
    transport: &Transport,
    correlation_id: &str,
    stage: PipelineStage,
    status: PipelineStatus,
    details: Value,
) -> Result<(), String> {
    publish_status(
        transport,
        &env().jobs_status_topic,
        correlation_id,
        stage,
        status,
        Some(details),
    )
    .await
}

// Reads the correlation id of a payload before validation, so a bad event can still be retried
pub(crate) fn raw_correlation_id(payload: &Value) -> Option<String> {
    payload
        .get("correlationId")
        .and_then(Value::as_str)
        .map(str::to_string)
}

// Everything the failure path of a write-model or read-model event needs
pub(crate) struct StageFailure<'a> {
    // Name of the stage in log lines
    pub stage: &'static str,

    // The event that failed
    pub event: &'a GatewayEvent,

    // Correlation id when it could be read
    pub correlation_id: Option<String>,

    // Why the event failed, without mail content or SQL text
    pub reason: String,

    // Text of the FAILED status
    pub message: &'static str,

    // Topic of the stage, retries go back here
    pub stage_topic: String,

    // Dead letter topic of the stage
    pub dlq_topic: String,

    // Retry count of the delivery that failed
    pub retry_count: u32,
}

// Failure path of SPEC section 8.2: FAILED status, retry or dead letter, ACK only after that publish worked
pub(crate) async fn handle_stage_failure(failure: StageFailure<'_>) {
    let StageFailure {
        stage,
        event,
        correlation_id,
        reason,
        message,
        stage_topic,
        dlq_topic,
        retry_count,
    } = failure;

    // Without a readable correlation id there is nothing to retry or acknowledge
    let Some(correlation_id) = correlation_id else {
        tracing::error!(
            target: LOG_TARGET,
            "{stage} failed without correlationId, not acknowledged: {reason}"
        );

        return;
    };

    tracing::error!(
        target: LOG_TARGET,
        "{stage} failed correlationId={correlation_id}: {reason}"
    );

    try_publish_failed_status(
        &event.transport,
        &env().jobs_status_topic,
        &correlation_id,
        message,
        &reason,
    )
    .await;

    let published = publish_retry_or_dlq(FailureContext {
        transport: &event.transport,
        stage_topic,
        dlq_topic,
        payload: event.payload.clone(),
        correlation_id: correlation_id.clone(),
        current_retry_count: retry_count,
        failure_reason: reason,
        original_partition: event.meta.partition,
        original_offset: event.meta.offset.clone(),
    })
    .await;

    // A failed retry or dead letter publish means no ACK, so the message is delivered again
    if let Err(error) = published {
        tracing::error!(
            target: LOG_TARGET,
            "{stage} retry or dead letter publish failed correlationId={correlation_id}: {error}"
        );

        return;
    }

    if let Err(error) = event.transport.send_processed(&correlation_id) {
        tracing::error!(
            target: LOG_TARGET,
            "{stage} ACK failed correlationId={correlation_id}: {error}"
        );
    }
}
