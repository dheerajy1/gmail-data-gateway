use std::{fmt::Display, sync::LazyLock, time::Duration};

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    config::env::env,
    errors::{api_error::ApiError, catalog::internal_server_error},
};

// Longest the gateway call may take before it counts as failed
const GATEWAY_TIMEOUT: Duration = Duration::from_secs(10);

// Shared http client, reuses connections between calls
static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(GATEWAY_TIMEOUT)
        .build()
        .expect("Failed to build the gateway http client")
});

// Where the Kafka gateway is and the four credentials it needs, no Debug because it holds secrets
pub struct GatewayTarget<'a> {
    // Base url of the gateway, a trailing slash is allowed
    pub base_url: &'a str,

    // Value of the x-client-id header
    pub client_id: &'a str,

    // Value of the x-client-secret header
    pub client_secret: &'a str,

    // Value of the x-api-key header
    pub api_key: &'a str,

    // Value of the x-api-secret header
    pub api_secret: &'a str,
}

// Transport envelope sent to the Kafka gateway, the gateway accepts only these three fields
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayPublishRequest<T: Serialize> {
    // Kafka topic the value is published to
    pub topic: String,

    // Uuid v7 that follows the event through the pipeline
    pub correlation_id: String,

    // Event published as the Kafka message value
    pub value: T,
}

// Success reply of the Kafka gateway
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayPublishResponse {
    // Always true on success
    pub success: bool,

    // Always 202 on success (D21)
    pub status_code: u16,

    // Gateway message
    pub message: String,

    // Topic and correlation id the gateway accepted
    pub data: GatewayPublishData,
}

// Data part of the gateway reply, read from kafka-api-gateway-rust (HttpPublishData)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayPublishData {
    // Topic the gateway accepted
    pub topic: String,

    // Correlation id the gateway accepted
    pub correlation_id: String,
}

// Logs the real gateway failure and returns the generic gateway error to the caller
fn gateway_error(detail: impl Display) -> ApiError {
    tracing::error!(target: "Gateway", "{detail}");

    ApiError::from_entry(internal_server_error::KAFKA_GATEWAY_FAILED)
}

// Publishes one transport envelope to the gateway named in the env and validates its reply
pub async fn post_to_gateway<T: Serialize>(
    request: &GatewayPublishRequest<T>,
) -> Result<GatewayPublishResponse, ApiError> {
    let env = env();

    let target = GatewayTarget {
        base_url: &env.onprem_server_url,
        client_id: &env.client_id,
        client_secret: &env.client_secret,
        api_key: &env.kafka_api_key,
        api_secret: &env.kafka_api_key_secret,
    };

    post_to_gateway_at(&target, request).await
}

// Publishes one transport envelope to the given gateway and validates its reply (SPEC section 7)
pub async fn post_to_gateway_at<T: Serialize>(
    target: &GatewayTarget<'_>,
    request: &GatewayPublishRequest<T>,
) -> Result<GatewayPublishResponse, ApiError> {
    let url = format!(
        "{}/api/v1/http-publish",
        target.base_url.trim_end_matches('/')
    );

    let response = CLIENT
        .post(url)
        .header("x-client-id", target.client_id)
        .header("x-client-secret", target.client_secret)
        .header("x-api-key", target.api_key)
        .header("x-api-secret", target.api_secret)
        .json(request)
        .send()
        .await
        .map_err(gateway_error)?;

    // Any non 2xx status is a failure
    if !response.status().is_success() {
        return Err(gateway_error(format!(
            "gateway returned status {}",
            response.status()
        )));
    }

    // A body that is not the expected json is a failure
    let parsed: GatewayPublishResponse = response
        .json()
        .await
        .map_err(|error| gateway_error(format!("response failed contract validation: {error}")))?;

    // The gateway must confirm success with status 202
    if !parsed.success || parsed.status_code != 202 {
        return Err(gateway_error(
            "response failed contract validation: unexpected success or statusCode",
        ));
    }

    tracing::info!(
        target: "KafkaPublisher",
        "accepted topic={} correlationId={}",
        request.topic,
        request.correlation_id
    );

    Ok(parsed)
}
