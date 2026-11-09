use jobs_api_gateway::config::env::env;
use serde_json::json;

use super::{LOG_TARGET, StageFailure, handle_stage_failure, raw_correlation_id, report};
use crate::{
    event::JobsEvent,
    gateway::GatewayEvent,
    postgres::execute_read_model,
    retry::get_retry_count,
    status::{PipelineStage, PipelineStatus},
};

// Handles one event of the read-model topic (SPEC section 8.2)
//
// Success: READ_MODEL_STARTED, PostgreSQL procedure, READ_MODEL_SUCCEEDED, COMPLETED, then ACK.
// Failure: FAILED status (best effort), retry or dead letter, ACK only after that publish worked.
pub async fn handle_read_model_event(event: GatewayEvent) {
    let retry_count = get_retry_count(&event.meta.headers);

    // Known before validation so a bad event can still be retried or dead lettered
    let mut correlation_id = raw_correlation_id(&event.payload);

    let Err(reason) = process(&event, retry_count, &mut correlation_id).await else {
        return;
    };

    handle_stage_failure(StageFailure {
        stage: "read-model",
        event: &event,
        correlation_id,
        reason,
        message: "Read-model processing failed",
        stage_topic: env().jobs_read_topic.clone(),
        dlq_topic: env().jobs_read_dlq_topic.clone(),
        retry_count,
    })
    .await;
}

// Runs the success path, any error returned here goes to the failure path
async fn process(
    event: &GatewayEvent,
    retry_count: u32,
    correlation_id: &mut Option<String>,
) -> Result<(), String> {
    let transport = &event.transport;

    // 1. Validate
    let parsed = JobsEvent::parse(&event.payload)?;

    *correlation_id = Some(parsed.correlation_id.clone());

    let id = parsed.correlation_id.as_str();

    // 2. READ_MODEL_STARTED
    report(
        transport,
        id,
        PipelineStage::ReadModelStarted,
        PipelineStatus::Started,
        json!({
            "message": "PostgreSQL read-model starting",
            "procedure": parsed.job.read_model,
            "retryCount": retry_count,
        }),
    )
    .await?;

    // 3. PostgreSQL procedure, the name comes from the event and is never hardcoded
    let applied = execute_read_model(&parsed.job.read_model, id, &parsed.records).await?;

    // 4. READ_MODEL_SUCCEEDED, a replay is a success
    report(
        transport,
        id,
        PipelineStage::ReadModelSucceeded,
        PipelineStatus::Succeeded,
        json!({
            "message": if applied.replayed {
                "PostgreSQL read model already applied (replay)"
            } else {
                "PostgreSQL read-model succeeded"
            },
            "rowsInserted": applied.rows_inserted,
            "replayed": applied.replayed,
        }),
    )
    .await?;

    // 5. COMPLETED
    report(
        transport,
        id,
        PipelineStage::Completed,
        PipelineStatus::Succeeded,
        json!({
            "message": "Pipeline completed successfully",
            "replayed": applied.replayed,
        }),
    )
    .await?;

    // 6. ACK
    transport.send_processed(id)?;

    tracing::info!(
        target: LOG_TARGET,
        "read-model ok correlationId={id} replayed={}",
        applied.replayed
    );

    Ok(())
}
