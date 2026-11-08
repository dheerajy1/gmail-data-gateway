use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use subscriber::{
    failure::{FailureAction, FailureContext, publish_retry_or_dlq},
    status::{
        PipelineStage, PipelineStatus, StatusEvent, publish_status, try_publish_failed_status,
    },
    transport::Transport,
};
use tokio::sync::mpsc;

// Frames the stub gateway received
type Frames = Arc<Mutex<Vec<Value>>>;

// Builds a transport with a stub gateway that records every frame and answers every publish
fn stub(fail: bool) -> (Arc<Transport>, Frames) {
    let (outbound, mut queue) = mpsc::unbounded_channel::<String>();
    let transport = Arc::new(Transport::new(outbound));
    let frames: Frames = Arc::new(Mutex::new(Vec::new()));

    let task_transport = Arc::clone(&transport);
    let task_frames = Arc::clone(&frames);

    tokio::spawn(async move {
        while let Some(text) = queue.recv().await {
            let Ok(frame) = serde_json::from_str::<Value>(&text) else {
                continue;
            };

            task_frames.lock().unwrap().push(frame.clone());

            if frame["type"] != "publish" {
                continue;
            }

            let reply = if fail {
                json!({ "type": "error", "requestId": frame["requestId"], "reason": "gateway said no" })
            } else {
                json!({ "type": "published", "requestId": frame["requestId"] })
            };

            task_transport.handle_control(&reply);
        }
    });

    (transport, frames)
}

// A failure context for the write stage with the given retry count
fn context(transport: &Transport, retry_count: u32) -> FailureContext<'_> {
    FailureContext {
        transport,
        stage_topic: "t-write".to_string(),
        dlq_topic: "t-write-dlq".to_string(),
        payload: json!({ "correlationId": "c1", "records": ["x"] }),
        correlation_id: "c1".to_string(),
        current_retry_count: retry_count,
        failure_reason: "boom".to_string(),
        original_partition: Some(2),
        original_offset: Some("99".to_string()),
    }
}

// The recorded frames
fn recorded(frames: &Frames) -> Vec<Value> {
    frames.lock().unwrap().clone()
}

// Counts 0, 1 and 2 are republished to the same topic with the count raised by one
#[tokio::test]
async fn retries_republish_to_the_same_topic() {
    for count in 0..3u32 {
        let (transport, frames) = stub(false);

        let action = publish_retry_or_dlq(context(&transport, count)).await;

        assert_eq!(action, Ok(FailureAction::Retry));

        let frame = recorded(&frames).remove(0);

        assert_eq!(frame["topic"], "t-write");
        assert_eq!(frame["key"], "c1");
        assert_eq!(
            frame["value"],
            json!({ "correlationId": "c1", "records": ["x"] })
        );
        assert_eq!(
            frame["headers"],
            json!({ "x-jobs-retry-count": (count + 1).to_string() })
        );
    }
}

// At the maximum the message goes to the dead letter topic with the five headers
#[tokio::test]
async fn dead_letter_carries_the_failure_headers() {
    for count in [3u32, 5] {
        let (transport, frames) = stub(false);

        let action = publish_retry_or_dlq(context(&transport, count)).await;

        assert_eq!(action, Ok(FailureAction::DeadLetter));

        let frame = recorded(&frames).remove(0);

        assert_eq!(frame["topic"], "t-write-dlq");
        assert_eq!(frame["key"], "c1");
        assert_eq!(
            frame["headers"],
            json!({
                "x-jobs-retry-count": count.to_string(),
                "x-original-topic": "t-write",
                "x-original-partition": "2",
                "x-original-offset": "99",
                "x-correlation-id": "c1",
                "x-failure-reason": "boom",
            })
        );
    }
}

// Partition and offset headers are left out when unknown or empty
#[tokio::test]
async fn dead_letter_leaves_out_unknown_partition_and_offset() {
    let (transport, frames) = stub(false);

    let mut ctx = context(&transport, 3);
    ctx.original_partition = None;
    ctx.original_offset = Some(String::new());

    assert_eq!(
        publish_retry_or_dlq(ctx).await,
        Ok(FailureAction::DeadLetter)
    );

    let headers = recorded(&frames).remove(0)["headers"].clone();

    assert!(headers.get("x-original-partition").is_none());
    assert!(headers.get("x-original-offset").is_none());
}

// A long failure reason is cut to 500 characters in the header
#[tokio::test]
async fn dead_letter_cuts_the_reason_to_500_characters() {
    let (transport, frames) = stub(false);

    let mut ctx = context(&transport, 3);
    ctx.failure_reason = "é".repeat(600);

    assert_eq!(
        publish_retry_or_dlq(ctx).await,
        Ok(FailureAction::DeadLetter)
    );

    let frame = recorded(&frames).remove(0);
    let reason = frame["headers"]["x-failure-reason"]
        .as_str()
        .unwrap_or_default();

    assert_eq!(reason.chars().count(), 500);
}

// A failed publish is returned to the caller for both a retry and a dead letter
#[tokio::test]
async fn publish_failure_is_returned() {
    for count in [0u32, 3] {
        let (transport, _frames) = stub(true);

        let action = publish_retry_or_dlq(context(&transport, count)).await;

        assert_eq!(action, Err("gateway said no".to_string()));
    }
}

// A status event has fresh UUIDv7 ids, the stage and status names, a UTC timestamp and a null sequence
#[tokio::test]
async fn status_event_is_valid_and_complete() {
    let (transport, frames) = stub(false);
    let correlation_id = "0190a3c0-7b1e-7000-8000-000000000001";

    let result = publish_status(
        &transport,
        "t-status",
        correlation_id,
        PipelineStage::AzureWriteStarted,
        PipelineStatus::Started,
        None,
    )
    .await;

    assert_eq!(result, Ok(()));

    let frame = recorded(&frames).remove(0);

    assert_eq!(frame["topic"], "t-status");
    assert_eq!(frame["key"], correlation_id);

    let value = &frame["value"];

    assert_eq!(value["stage"], "AZURE_WRITE_STARTED");
    assert_eq!(value["status"], "STARTED");
    assert_eq!(value["sequence"], Value::Null);
    assert!(value.get("details").is_none());
    assert!(
        value["timestamp"]
            .as_str()
            .unwrap_or_default()
            .ends_with('Z')
    );

    // The published value passes the same validation the status consumer uses
    assert!(StatusEvent::parse(value).is_ok());
}

// Every status publish gets its own event id
#[tokio::test]
async fn each_status_event_gets_a_new_event_id() {
    let (transport, frames) = stub(false);
    let correlation_id = "0190a3c0-7b1e-7000-8000-000000000001";

    for _ in 0..2 {
        let result = publish_status(
            &transport,
            "t-status",
            correlation_id,
            PipelineStage::Received,
            PipelineStatus::Started,
            Some(json!({ "records": 1 })),
        )
        .await;

        assert_eq!(result, Ok(()));
    }

    let recorded = recorded(&frames);

    assert_ne!(
        recorded[0]["value"]["eventId"],
        recorded[1]["value"]["eventId"]
    );
    assert_eq!(recorded[0]["value"]["details"], json!({ "records": 1 }));
}

// A FAILED status carries the message and the error cut to 500 characters
#[tokio::test]
async fn failed_status_cuts_the_error_and_carries_the_message() {
    let (transport, frames) = stub(false);

    try_publish_failed_status(
        &transport,
        "t-status",
        "0190a3c0-7b1e-7000-8000-000000000001",
        "Azure write failed",
        &"e".repeat(700),
    )
    .await;

    let value = recorded(&frames).remove(0)["value"].clone();

    assert_eq!(value["stage"], "FAILED");
    assert_eq!(value["status"], "FAILED");
    assert_eq!(value["details"]["message"], "Azure write failed");
    assert_eq!(
        value["details"]["error"]
            .as_str()
            .unwrap_or_default()
            .chars()
            .count(),
        500
    );
}

// A failed status publish is only logged, it never panics or returns an error
#[tokio::test]
async fn failed_status_publish_failure_is_swallowed() {
    let (transport, _frames) = stub(true);

    try_publish_failed_status(&transport, "t-status", "c1", "message", "error").await;
}
