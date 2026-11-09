use std::{collections::HashMap, future::Future, sync::Arc, time::Duration};

use futures_util::{SinkExt, StreamExt};
use jobs_api_gateway::config::env::env;
use serde_json::{Map, Value, json};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{
        Message, client::IntoClientRequest, handshake::client::Request, http::HeaderValue,
    },
};

use crate::{retry::RETRY_COUNT_HEADER, transport::Transport};

// Log target of the gateway connection
const LOG_TARGET: &str = "JobsSub";

// Metadata that comes with an inbound event
#[derive(Debug, Clone)]
pub struct EventMeta {
    // Kafka headers of the message, string values only
    pub headers: HashMap<String, String>,

    // Partition the message was read from
    pub partition: Option<i64>,

    // Offset of the message, kept as text
    pub offset: Option<String>,
}

// One event delivered by the gateway, handed to the handler
pub struct GatewayEvent {
    // Topic the event was consumed from
    pub topic: String,

    // Event body, the message without the control fields
    pub payload: Value,

    // Headers, partition and offset of the event
    pub meta: EventMeta,

    // Transport of the connection that delivered the event, used to publish and acknowledge
    pub transport: Arc<Transport>,
}

// Connection values of the gateway, the secrets are never logged
pub struct GatewayConfig {
    // Full websocket url including the subscribe path
    pub url: String,

    // Value of the x-client-id header
    pub client_id: String,

    // Value of the x-client-secret header
    pub client_secret: String,

    // Value of the x-api-key header
    pub api_key: String,

    // Value of the x-api-secret header
    pub api_secret: String,

    // Value of the x-subscriber-id header
    pub subscriber_id: String,
}

impl GatewayConfig {
    // Builds the config from the env, the subscriber id is SOURCE_CLIENT_ID (SPEC section 8.1)
    pub fn from_env() -> Self {
        let env = env();

        Self {
            url: websocket_url(&env.onprem_server_url),
            client_id: env.client_id.clone(),
            client_secret: env.client_secret.clone(),
            api_key: env.kafka_api_key.clone(),
            api_secret: env.kafka_api_key_secret.clone(),
            subscriber_id: env.source_client_id.clone(),
        }
    }
}

// Reconnect delays: the first delay is base and it doubles up to max
#[derive(Debug, Clone, Copy)]
pub struct Backoff {
    // First reconnect delay
    pub base: Duration,

    // Longest reconnect delay
    pub max: Duration,
}

impl Backoff {
    // 2 seconds doubling up to 30 seconds (SPEC section 8.1)
    pub const STANDARD: Self = Self {
        base: Duration::from_secs(2),
        max: Duration::from_secs(30),
    };

    // Delay before the reconnect with the given attempt number, the first attempt is 0
    pub fn delay(&self, attempt: u32) -> Duration {
        self.max.min(self.base * 2u32.pow(attempt.min(4)))
    }
}

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

// Turns the http gateway url into the websocket url, http becomes ws and https becomes wss
pub fn websocket_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');

    let base = match base.strip_prefix("http") {
        Some(rest) => format!("ws{rest}"),
        None => base.to_string(),
    };

    format!("{base}/api/v1/ws-subscribe")
}

// Builds the websocket handshake request with the five gateway headers
fn build_request(config: &GatewayConfig) -> Result<Request, String> {
    let mut request = config
        .url
        .as_str()
        .into_client_request()
        .map_err(|error| error.to_string())?;

    let headers = request.headers_mut();

    for (name, value) in [
        ("x-client-id", &config.client_id),
        ("x-client-secret", &config.client_secret),
        ("x-api-key", &config.api_key),
        ("x-api-secret", &config.api_secret),
        ("x-subscriber-id", &config.subscriber_id),
    ] {
        let value = HeaderValue::from_str(value).map_err(|error| error.to_string())?;

        headers.insert(name, value);
    }

    Ok(request)
}

// Connects to the gateway and keeps reconnecting with backoff, runs until the future is dropped
pub async fn run<F, Fut>(config: GatewayConfig, topics: Vec<String>, handler: F, backoff: Backoff)
where
    F: Fn(GatewayEvent) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let handler = Arc::new(handler);
    let mut attempt: u32 = 0;

    loop {
        match build_request(&config) {
            Ok(request) => {
                tracing::info!(target: LOG_TARGET, "connect {}", request.uri());

                match connect_async(request).await {
                    Ok((socket, _)) => {
                        attempt = 0;

                        tracing::info!(target: LOG_TARGET, "WS connected");

                        run_session(socket, &topics, &handler).await;

                        tracing::warn!(target: LOG_TARGET, "WS closed");
                    }
                    Err(error) => {
                        tracing::error!(target: LOG_TARGET, "WS connect failed: {error}");
                    }
                }
            }
            Err(error) => {
                tracing::error!(target: LOG_TARGET, "invalid gateway request: {error}");
            }
        }

        let delay = backoff.delay(attempt);

        attempt += 1;

        tracing::warn!(
            target: LOG_TARGET,
            "reconnect attempt={attempt} in={}ms",
            delay.as_millis()
        );

        tokio::time::sleep(delay).await;
    }
}

// Runs one connection: subscribes, sends queued frames and reads inbound frames until it closes
async fn run_session<F, Fut>(mut socket: Socket, topics: &[String], handler: &Arc<F>)
where
    F: Fn(GatewayEvent) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (outbound, mut queue) = mpsc::unbounded_channel::<String>();
    let transport = Arc::new(Transport::new(outbound.clone()));

    // One subscribe frame per topic
    for topic in topics {
        let _ = outbound.send(json!({ "type": "subscribe", "topic": topic }).to_string());
    }

    loop {
        tokio::select! {
            // Frames queued by the transport go out to the socket
            frame = queue.recv() => {
                let Some(frame) = frame else { break };

                if socket.send(Message::text(frame)).await.is_err() {
                    break;
                }
            }
            // Frames from the gateway
            incoming = socket.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        handle_text(text.as_str(), &transport, handler);
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(error)) => {
                        tracing::error!(target: LOG_TARGET, "WS error: {error}");

                        break;
                    }
                }
            }
        }
    }

    // Anything still waiting for a publish answer fails now
    transport.reject_all("WebSocket closed");
}

// Routes one text frame: control answers go to the transport, events go to the handler
fn handle_text<F, Fut>(text: &str, transport: &Arc<Transport>, handler: &Arc<F>)
where
    F: Fn(GatewayEvent) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let Ok(message) = serde_json::from_str::<Value>(text) else {
        tracing::error!(target: LOG_TARGET, "message is not json");

        return;
    };

    let kind = message.get("type").and_then(Value::as_str).unwrap_or("");
    let has_request_id = message
        .get("requestId")
        .is_some_and(|value| !value.is_null());

    if kind == "published" || (kind == "error" && has_request_id) {
        transport.handle_control(&message);

        return;
    }

    match kind {
        "ready" => {
            tracing::info!(target: LOG_TARGET, "control ready");
        }
        "subscribed" => {
            // Read before the log call, the tracing macros bring their own Value trait into scope
            let topic = message.get("topic").and_then(Value::as_str).unwrap_or("?");

            tracing::info!(target: LOG_TARGET, "control subscribed topic={topic}");
        }
        "event" => {
            let Some(event) = parse_event(&message, transport) else {
                return;
            };

            let handler = Arc::clone(handler);

            // Each event runs on its own task, like the old promise per message
            tokio::spawn(async move {
                handler(event).await;
            });
        }
        // The gateway confirms that a processed ACK was accepted, nothing to do
        "ack-received" => {}
        "error" => {
            // Read before the log call, the tracing macros bring their own Value trait into scope
            let reason = message
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("without message");

            tracing::error!(target: LOG_TARGET, "control error {reason}");
        }
        other => {
            // Only the frame type is logged, an unknown frame may carry mail content (rule 4)
            tracing::warn!(
                target: LOG_TARGET,
                "control unknown type={}",
                if other.is_empty() { "none" } else { other }
            );
        }
    }
}

// Reads the topic, headers, partition and offset out of an event frame, the rest is the payload
fn parse_event(message: &Value, transport: &Arc<Transport>) -> Option<GatewayEvent> {
    let correlation_id = message
        .get("correlationId")
        .and_then(Value::as_str)
        .unwrap_or("?");

    let topic = match message.get("topic").and_then(Value::as_str) {
        Some(topic) if !topic.is_empty() => topic.to_string(),
        _ => {
            tracing::error!(
                target: LOG_TARGET,
                "invalid event payload correlationId={correlation_id}"
            );

            return None;
        }
    };

    let headers: HashMap<String, String> = message
        .get("headers")
        .and_then(Value::as_object)
        .map(|raw| {
            raw.iter()
                .filter_map(|(key, value)| Some((key.clone(), value.as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default();

    let partition = message.get("partition").and_then(Value::as_i64);

    let offset = match message.get("offset") {
        Some(Value::String(offset)) => Some(offset.clone()),
        Some(Value::Number(offset)) => Some(offset.to_string()),
        _ => None,
    };

    // The payload is the frame without the control fields
    let mut payload: Map<String, Value> = message.as_object().cloned().unwrap_or_default();

    for key in ["type", "topic", "headers", "partition", "offset"] {
        payload.remove(key);
    }

    tracing::info!(
        target: LOG_TARGET,
        "event topic={topic} correlationId={correlation_id} retry={}",
        headers
            .get(RETRY_COUNT_HEADER)
            .map(String::as_str)
            .unwrap_or("0")
    );

    Some(GatewayEvent {
        topic,
        payload: Value::Object(payload),
        meta: EventMeta {
            headers,
            partition,
            offset,
        },
        transport: Arc::clone(transport),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // A transport whose frames nobody reads
    fn transport() -> Arc<Transport> {
        let (outbound, _queue) = mpsc::unbounded_channel();

        Arc::new(Transport::new(outbound))
    }

    // http becomes ws, https becomes wss, and a trailing slash is dropped
    #[test]
    fn websocket_url_follows_the_scheme() {
        assert_eq!(
            websocket_url("http://host:8080"),
            "ws://host:8080/api/v1/ws-subscribe"
        );
        assert_eq!(
            websocket_url("https://host/"),
            "wss://host/api/v1/ws-subscribe"
        );
    }

    // The reconnect delay doubles from the base and stops at the maximum
    #[test]
    fn backoff_doubles_up_to_the_maximum() {
        let delays: Vec<u64> = (0..7)
            .map(|attempt| Backoff::STANDARD.delay(attempt).as_secs())
            .collect();

        assert_eq!(delays, [2, 4, 8, 16, 30, 30, 30]);
    }

    // An event frame is split into topic, headers, partition, offset and payload
    #[test]
    fn parse_event_splits_control_fields_from_the_payload() {
        let message = json!({
            "type": "event",
            "topic": "t-write",
            "headers": { "x-jobs-retry-count": "2", "number": 5 },
            "partition": 3,
            "offset": 42,
            "correlationId": "c1",
            "records": [],
        });

        let Some(event) = parse_event(&message, &transport()) else {
            panic!("event rejected");
        };

        assert_eq!(event.topic, "t-write");
        assert_eq!(
            event.payload,
            json!({ "correlationId": "c1", "records": [] })
        );
        assert_eq!(
            event.meta.headers,
            HashMap::from([("x-jobs-retry-count".to_string(), "2".to_string())])
        );
        assert_eq!(event.meta.partition, Some(3));
        assert_eq!(event.meta.offset.as_deref(), Some("42"));
    }

    // A string offset is kept and a frame without a topic is rejected
    #[test]
    fn parse_event_handles_string_offsets_and_missing_topics() {
        let with_offset = json!({ "type": "event", "topic": "t", "offset": "7" });

        let Some(event) = parse_event(&with_offset, &transport()) else {
            panic!("event rejected");
        };

        assert_eq!(event.meta.offset.as_deref(), Some("7"));

        assert!(parse_event(&json!({ "type": "event" }), &transport()).is_none());
        assert!(parse_event(&json!({ "type": "event", "topic": "" }), &transport()).is_none());
    }
}
