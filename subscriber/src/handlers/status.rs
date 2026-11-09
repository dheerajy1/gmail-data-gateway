use serde::Serialize;
use serde_json::Value;

use super::LOG_TARGET;
use crate::{gateway::GatewayEvent, postgres::apply_status, status::StatusEvent};

// Turns a stage or status into the text the database stores, such as AZURE_WRITE_STARTED
fn as_text<T: Serialize>(value: &T) -> Result<String, String> {
    match serde_json::to_value(value).map_err(|error| error.to_string())? {
        Value::String(text) => Ok(text),
        _ => Err("stage or status is not text".to_string()),
    }
}

// Handles one event of the status topic (SPEC section 8.2), it only saves pipeline state and history
//
// The event id from the payload is the idempotency key. A failure sends no ACK and
// publishes no retry or dead letter, so the gateway and Kafka deliver the event again.
pub async fn handle_status_event(event: GatewayEvent) {
    // Known once the payload is valid, so the failure log can name the event
    let mut ids: Option<(String, String)> = None;

    let result = async {
        let status_event = StatusEvent::parse(&event.payload)?;

        ids = Some((
            status_event.event_id.clone(),
            status_event.correlation_id.clone(),
        ));

        let stage = as_text(&status_event.stage)?;
        let status = as_text(&status_event.status)?;

        let applied = apply_status(
            &status_event.event_id,
            &status_event.correlation_id,
            &stage,
            &status,
            &status_event.timestamp,
            status_event.details.as_ref(),
        )
        .await?;

        event
            .transport
            .send_processed(&status_event.correlation_id)?;

        tracing::info!(
            target: LOG_TARGET,
            "status ok eventId={} correlationId={} stage={stage} sequence={} replayed={}",
            status_event.event_id,
            status_event.correlation_id,
            applied.assigned_sequence,
            applied.replayed
        );

        Ok::<(), String>(())
    }
    .await;

    if let Err(error) = result {
        let (event_id, correlation_id) = ids.unzip();

        tracing::error!(
            target: LOG_TARGET,
            "status failed, not acknowledged eventId={} correlationId={}: {error}",
            event_id.as_deref().unwrap_or("?"),
            correlation_id.as_deref().unwrap_or("?")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::{PipelineStage, PipelineStatus};

    // Stages and statuses are stored as their SCREAMING_SNAKE_CASE names
    #[test]
    fn stage_and_status_become_database_text() {
        assert_eq!(
            as_text(&PipelineStage::AzureWriteStarted),
            Ok("AZURE_WRITE_STARTED".to_string())
        );
        assert_eq!(
            as_text(&PipelineStage::Completed),
            Ok("COMPLETED".to_string())
        );
        assert_eq!(
            as_text(&PipelineStatus::Succeeded),
            Ok("SUCCEEDED".to_string())
        );
    }
}
