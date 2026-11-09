use jobs_api_gateway::config::env::env;
use serde_json::json;

use super::{LOG_TARGET, StageFailure, handle_stage_failure, raw_correlation_id, report};
use crate::{
    azure::execute_write_model,
    event::JobsEvent,
    gateway::GatewayEvent,
    retry::get_retry_count,
    status::{PipelineStage, PipelineStatus},
    transport::PublishRequest,
};

// Handles one event of the write-model topic (SPEC section 8.2)
//
// Success: RECEIVED, AZURE_WRITE_STARTED, Azure procedure, AZURE_WRITE_SUCCEEDED,
// publish to the read-model topic, then ACK.
// Failure: FAILED status (best effort), retry or dead letter, ACK only after that publish worked.
pub async fn handle_write_model_event(event: GatewayEvent) {
    let retry_count = get_retry_count(&event.meta.headers);

    // Known before validation so a bad event can still be retried or dead lettered
    let mut correlation_id = raw_correlation_id(&event.payload);

    let Err(reason) = process(&event, retry_count, &mut correlation_id).await else {
        return;
    };

    handle_stage_failure(StageFailure {
        stage: "write-model",
        event: &event,
        correlation_id,
        reason,
        message: "Write-model processing failed",
        stage_topic: env().jobs_write_topic.clone(),
        dlq_topic: env().jobs_write_dlq_topic.clone(),
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

    // 2. RECEIVED
    report(
        transport,
        id,
        PipelineStage::Received,
        PipelineStatus::Succeeded,
        json!({
            "message": "Valid write-model event accepted by subscriber",
            "sourceClientId": parsed.source_client_id,
            "recordCount": parsed.records.len(),
            "retryCount": retry_count,
        }),
    )
    .await?;

    // 3. AZURE_WRITE_STARTED
    report(
        transport,
        id,
        PipelineStage::AzureWriteStarted,
        PipelineStatus::Started,
        json!({
            "message": "Azure write-model starting",
            "procedure": parsed.job.write_model,
            "recordCount": parsed.records.len(),
        }),
    )
    .await?;

    // 4. Azure procedure, the name comes from the event and is never hardcoded
    let notes = execute_write_model(&parsed.job.write_model, id, &parsed.records).await?;

    // 5. AZURE_WRITE_SUCCEEDED, a replay is a success
    report(
        transport,
        id,
        PipelineStage::AzureWriteSucceeded,
        PipelineStatus::Succeeded,
        json!({
            "message": if notes.replayed {
                "Azure write already completed (replay)"
            } else {
                "Azure write-model succeeded"
            },
            "rowsInserted": notes.rows_inserted,
            "replayed": notes.replayed,
        }),
    )
    .await?;

    // 6. Next stage, the same event goes to the read-model topic
    transport
        .publish(PublishRequest {
            topic: env().jobs_read_topic.clone(),
            key: Some(id.to_string()),
            value: event.payload.clone(),
            headers: Default::default(),
        })
        .await?;

    // 7. ACK only after the read-model publish succeeded
    transport.send_processed(id)?;

    tracing::info!(
        target: LOG_TARGET,
        "write-model ok correlationId={id} replayed={}",
        notes.replayed
    );

    Ok(())
}
