use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::{
    event::{describe_shape, is_uuid_v7},
    transport::{PublishRequest, Transport},
};

// Longest error text stored in the details of a failed status (SPEC section 8.5)
const MAX_ERROR_LENGTH: usize = 500;

// Pipeline stage a status event reports on (SPEC section 6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PipelineStage {
    // The subscriber received the event
    Received,

    // The Azure write started
    AzureWriteStarted,

    // The Azure write finished
    AzureWriteSucceeded,

    // The PostgreSQL read model started
    ReadModelStarted,

    // The PostgreSQL read model finished
    ReadModelSucceeded,

    // The whole pipeline finished
    Completed,

    // A stage failed
    Failed,
}

// Outcome a status event reports (SPEC section 6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PipelineStatus {
    // The stage started
    Started,

    // The stage finished successfully
    Succeeded,

    // The stage failed
    Failed,
}

// A status event as consumed from the status topic (SPEC section 6)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEvent {
    // UUIDv7 of this status event
    pub event_id: String,

    // UUIDv7 of the chunk the status belongs to
    pub correlation_id: String,

    // Stage that reports
    pub stage: PipelineStage,

    // Outcome of the stage
    pub status: PipelineStatus,

    // ISO timestamp of the status
    pub timestamp: String,

    // Sequence number, the database assigns it so it is normally null
    #[serde(default)]
    pub sequence: Option<f64>,

    // Optional object with extra information
    #[serde(default)]
    pub details: Option<Value>,
}

impl StatusEvent {
    // Validates a payload and returns the status event, the error never contains mail content
    pub fn parse(payload: &Value) -> Result<Self, String> {
        let event: Self = serde_json::from_value(payload.clone())
            .map_err(|error| format!("invalid status event: {}", describe_shape(&error)))?;

        if !is_uuid_v7(&event.event_id) {
            return Err("invalid status event: eventId must be a uuid v7".to_string());
        }

        if !is_uuid_v7(&event.correlation_id) {
            return Err("invalid status event: correlationId must be a uuid v7".to_string());
        }

        if event.timestamp.chars().count() < 10 {
            return Err("invalid status event: timestamp is too short".to_string());
        }

        if event
            .details
            .as_ref()
            .is_some_and(|details| !details.is_object())
        {
            return Err("invalid status event: details must be an object".to_string());
        }

        Ok(event)
    }
}

// Publishes one status event to the status topic and waits for the gateway to confirm it
pub async fn publish_status(
    transport: &Transport,
    status_topic: &str,
    correlation_id: &str,
    stage: PipelineStage,
    status: PipelineStatus,
    details: Option<Value>,
) -> Result<(), String> {
    let mut value = Map::new();
    value.insert("eventId".into(), json!(Uuid::now_v7().to_string()));
    value.insert("correlationId".into(), json!(correlation_id));
    value.insert("stage".into(), json!(stage));
    value.insert("status".into(), json!(status));
    value.insert(
        "timestamp".into(),
        json!(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
    );
    value.insert("sequence".into(), Value::Null);

    if let Some(details) = details {
        value.insert("details".into(), details);
    }

    transport
        .publish(PublishRequest {
            topic: status_topic.to_string(),
            key: Some(correlation_id.to_string()),
            value: Value::Object(value),
            headers: Default::default(),
        })
        .await
}

// Publishes a FAILED status for a failure and only logs when that publish fails too (SPEC section 8.5)
pub async fn try_publish_failed_status(
    transport: &Transport,
    status_topic: &str,
    correlation_id: &str,
    message: &str,
    error: &str,
) {
    let details = json!({
        "message": message,
        "error": error.chars().take(MAX_ERROR_LENGTH).collect::<String>(),
    });

    if let Err(publish_error) = publish_status(
        transport,
        status_topic,
        correlation_id,
        PipelineStage::Failed,
        PipelineStatus::Failed,
        Some(details),
    )
    .await
    {
        tracing::error!(
            target: "JobsSub",
            "failed to publish FAILED status correlationId={correlation_id}: {publish_error}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // UUIDv7 ids used by the tests
    const EVENT_ID: &str = "0190a3c0-7b1e-7000-8000-000000000002";
    const CORRELATION_ID: &str = "0190a3c0-7b1e-7000-8000-000000000001";

    // A valid status payload
    fn valid() -> Value {
        json!({
            "eventId": EVENT_ID,
            "correlationId": CORRELATION_ID,
            "stage": "AZURE_WRITE_STARTED",
            "status": "STARTED",
            "timestamp": "2026-11-08T10:00:00.000Z",
            "sequence": null,
        })
    }

    // Parses the payload and returns the error text, panics when it is valid
    fn error_of(payload: &Value) -> String {
        match StatusEvent::parse(payload) {
            Ok(_) => panic!("payload should be invalid"),
            Err(error) => error,
        }
    }

    // A valid payload parses
    #[test]
    fn parses_a_valid_status_event() {
        let Ok(event) = StatusEvent::parse(&valid()) else {
            panic!("valid status rejected");
        };

        assert_eq!(event.stage, PipelineStage::AzureWriteStarted);
        assert_eq!(event.status, PipelineStatus::Started);
        assert_eq!(event.sequence, None);
        assert!(event.details.is_none());
    }

    // The sequence may be a number and details an object
    #[test]
    fn accepts_a_sequence_and_details_object() {
        let mut payload = valid();
        payload["sequence"] = json!(12);
        payload["details"] = json!({ "message": "ok" });

        let Ok(event) = StatusEvent::parse(&payload) else {
            panic!("status rejected");
        };

        assert_eq!(event.sequence, Some(12.0));
        assert!(event.details.is_some());
    }

    // Ids must be UUIDv7
    #[test]
    fn rejects_ids_that_are_not_uuid_v7() {
        let mut payload = valid();
        payload["eventId"] = json!("550e8400-e29b-41d4-a716-446655440000");

        assert!(error_of(&payload).contains("eventId"));

        let mut payload = valid();
        payload["correlationId"] = json!("nope");

        assert!(error_of(&payload).contains("correlationId"));
    }

    // A short timestamp and non-object details are rejected
    #[test]
    fn rejects_short_timestamp_and_non_object_details() {
        let mut payload = valid();
        payload["timestamp"] = json!("2026-11-0");

        assert!(error_of(&payload).contains("timestamp"));

        let mut payload = valid();
        payload["details"] = json!("text");

        assert!(error_of(&payload).contains("details"));
    }

    // An unknown stage or status is rejected
    #[test]
    fn rejects_unknown_stage_and_status() {
        let mut payload = valid();
        payload["stage"] = json!("NOPE");

        assert!(error_of(&payload).contains("unknown variant"));

        let mut payload = valid();
        payload["status"] = json!("NOPE");

        assert!(error_of(&payload).contains("unknown variant"));
    }

    // The enums serialize to the screaming snake case names of the contract
    #[test]
    fn enums_serialize_as_contract_names() {
        let stages = [
            (PipelineStage::Received, "RECEIVED"),
            (PipelineStage::AzureWriteStarted, "AZURE_WRITE_STARTED"),
            (PipelineStage::AzureWriteSucceeded, "AZURE_WRITE_SUCCEEDED"),
            (PipelineStage::ReadModelStarted, "READ_MODEL_STARTED"),
            (PipelineStage::ReadModelSucceeded, "READ_MODEL_SUCCEEDED"),
            (PipelineStage::Completed, "COMPLETED"),
            (PipelineStage::Failed, "FAILED"),
        ];

        for (stage, name) in stages {
            assert_eq!(json!(stage), name);
        }

        assert_eq!(json!(PipelineStatus::Started), "STARTED");
        assert_eq!(json!(PipelineStatus::Succeeded), "SUCCEEDED");
        assert_eq!(json!(PipelineStatus::Failed), "FAILED");
    }
}
