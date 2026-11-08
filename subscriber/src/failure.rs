use std::collections::HashMap;

use serde_json::Value;

use crate::{
    retry::{
        HEADER_CORRELATION_ID, HEADER_FAILURE_REASON, HEADER_ORIGINAL_OFFSET,
        HEADER_ORIGINAL_PARTITION, HEADER_ORIGINAL_TOPIC, RETRY_COUNT_HEADER, should_dlq,
    },
    transport::{PublishRequest, Transport},
};

// Longest failure reason put into the dead letter header (SPEC section 8.3)
const MAX_REASON_LENGTH: usize = 500;

// What was done with a failed message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureAction {
    // Republished to the same topic with the retry count raised
    Retry,

    // Published to the dead letter topic
    DeadLetter,
}

// Everything needed to retry a failed message or send it to the dead letter topic
pub struct FailureContext<'a> {
    // Transport that publishes
    pub transport: &'a Transport,

    // Topic the message came from, retries go back here
    pub stage_topic: String,

    // Dead letter topic of the stage
    pub dlq_topic: String,

    // The original payload, republished unchanged
    pub payload: Value,

    // Correlation id, used as the message key
    pub correlation_id: String,

    // Retry count of the delivery that failed
    pub current_retry_count: u32,

    // Why the delivery failed, the caller keeps mail content and SQL out of it
    pub failure_reason: String,

    // Partition the message came from
    pub original_partition: Option<i64>,

    // Offset the message came from
    pub original_offset: Option<String>,
}

// Retries the message while it has retries left, otherwise sends it to the dead letter topic (SPEC section 8.3)
pub async fn publish_retry_or_dlq(ctx: FailureContext<'_>) -> Result<FailureAction, String> {
    let reason: String = ctx.failure_reason.chars().take(MAX_REASON_LENGTH).collect();

    if !should_dlq(ctx.current_retry_count) {
        let next = ctx.current_retry_count + 1;

        let headers = HashMap::from([(RETRY_COUNT_HEADER.to_string(), next.to_string())]);

        tracing::warn!(
            target: "JobsSub",
            "retry correlationId={} attempt={next} topic={}",
            ctx.correlation_id,
            ctx.stage_topic
        );

        ctx.transport
            .publish(PublishRequest {
                topic: ctx.stage_topic,
                key: Some(ctx.correlation_id),
                value: ctx.payload,
                headers,
            })
            .await?;

        return Ok(FailureAction::Retry);
    }

    let mut headers = HashMap::from([
        (
            RETRY_COUNT_HEADER.to_string(),
            ctx.current_retry_count.to_string(),
        ),
        (HEADER_ORIGINAL_TOPIC.to_string(), ctx.stage_topic.clone()),
        (
            HEADER_CORRELATION_ID.to_string(),
            ctx.correlation_id.clone(),
        ),
        (HEADER_FAILURE_REASON.to_string(), reason),
    ]);

    if let Some(partition) = ctx.original_partition {
        headers.insert(HEADER_ORIGINAL_PARTITION.to_string(), partition.to_string());
    }

    if let Some(offset) = ctx.original_offset.filter(|offset| !offset.is_empty()) {
        headers.insert(HEADER_ORIGINAL_OFFSET.to_string(), offset);
    }

    tracing::error!(
        target: "JobsSub",
        "dead letter correlationId={} retries={} topic={}",
        ctx.correlation_id,
        ctx.current_retry_count,
        ctx.dlq_topic
    );

    ctx.transport
        .publish(PublishRequest {
            topic: ctx.dlq_topic,
            key: Some(ctx.correlation_id),
            value: ctx.payload,
            headers,
        })
        .await?;

    Ok(FailureAction::DeadLetter)
}
