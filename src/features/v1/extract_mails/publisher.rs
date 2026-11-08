use serde::Serialize;
use uuid::Uuid;

use super::reader::MailRecord;
use crate::{
    clients::kafka_gateway::{GatewayPublishRequest, GatewayTarget, post_to_gateway_at},
    errors::api_error::ApiError,
};

// Records per published event (SPEC section 6)
pub const CHUNK_SIZE: usize = 100;

// Values of the env that go into every event, tests pass their own
#[derive(Debug, Clone, Copy)]
pub struct PublishConfig<'a> {
    // Kafka topic of the write model events
    pub topic: &'a str,

    // Stored procedure of the write model
    pub write_model: &'a str,

    // Stored procedure of the read model
    pub read_model: &'a str,

    // Id of this client in the event
    pub source_client_id: &'a str,
}

// The two models an event is processed by
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JobModels<'a> {
    write_model: &'a str,
    read_model: &'a str,
}

// The jobs event, the value inside the gateway envelope (SPEC section 6)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JobsEvent<'a> {
    correlation_id: &'a str,
    job: JobModels<'a>,
    source_client_id: &'a str,
    records: &'a [MailRecord],
}

// Publishes one chunk as one event and checks the gateway reply
async fn publish_chunk(
    chunk: &[MailRecord],
    correlation_id: &str,
    config: &PublishConfig<'_>,
    gateway: &GatewayTarget<'_>,
) -> Result<(), ApiError> {
    let event = JobsEvent {
        correlation_id,
        job: JobModels {
            write_model: config.write_model,
            read_model: config.read_model,
        },
        source_client_id: config.source_client_id,
        records: chunk,
    };

    let request = GatewayPublishRequest {
        topic: config.topic.to_string(),
        correlation_id: correlation_id.to_string(),
        value: event,
    };

    post_to_gateway_at(gateway, &request).await?;

    Ok(())
}

// Publishes the records as events of at most CHUNK_SIZE records, oldest chunk first (D10)
// The records come newest first from Gmail, so the order is reversed first
// The first failed chunk stops the run and the error says how many chunks were already published (D11)
pub async fn publish_chunks(
    mut records: Vec<MailRecord>,
    config: &PublishConfig<'_>,
    gateway: &GatewayTarget<'_>,
) -> Result<Vec<String>, ApiError> {
    records.reverse();

    let total_chunks = records.len().div_ceil(CHUNK_SIZE);
    let mut correlation_ids = Vec::with_capacity(total_chunks);

    for chunk in records.chunks(CHUNK_SIZE) {
        let correlation_id = Uuid::now_v7().to_string();

        if let Err(error) = publish_chunk(chunk, &correlation_id, config, gateway).await {
            let message = format!(
                "{} Chunks already published: {} of {}.",
                error.message(),
                correlation_ids.len(),
                total_chunks
            );

            tracing::error!(
                target: "ExtractMails",
                "chunk {} of {} failed to publish",
                correlation_ids.len() + 1,
                total_chunks
            );

            return Err(error.with_message(message));
        }

        correlation_ids.push(correlation_id);
    }

    tracing::info!(target: "ExtractMails", "published {} chunks", correlation_ids.len());

    Ok(correlation_ids)
}
