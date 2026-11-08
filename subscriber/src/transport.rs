use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use serde_json::{Map, Value, json};
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

// Log target of the transport
const LOG_TARGET: &str = "WsTransport";

// Longest a publish may wait for the gateway to confirm it (SPEC section 8.1)
const PUBLISH_TIMEOUT: Duration = Duration::from_secs(30);

// One message to publish through the gateway websocket
#[derive(Debug, Clone)]
pub struct PublishRequest {
    // Topic to publish to
    pub topic: String,

    // Optional message key
    pub key: Option<String>,

    // Message value
    pub value: Value,

    // Optional Kafka headers, such as the retry count
    pub headers: HashMap<String, String>,
}

// Waiting publish, resolved when the gateway answers with the same request id
type Pending = oneshot::Sender<Result<(), String>>;

// Publish and ack channel of one websocket connection
pub struct Transport {
    // Queue of text frames the writer task sends to the socket
    outbound: mpsc::UnboundedSender<String>,

    // Publishes waiting for their gateway answer, keyed by request id
    pending: Mutex<HashMap<String, Pending>>,
}

impl Transport {
    // Creates the transport around the outbound frame queue of a connection
    pub fn new(outbound: mpsc::UnboundedSender<String>) -> Self {
        Self {
            outbound,
            pending: Mutex::new(HashMap::new()),
        }
    }

    // Publishes through the websocket and waits for the matching published or error answer
    pub async fn publish(&self, request: PublishRequest) -> Result<(), String> {
        self.publish_with_timeout(request, PUBLISH_TIMEOUT).await
    }

    // Same as publish with a chosen wait time, tests use a short one
    pub async fn publish_with_timeout(
        &self,
        request: PublishRequest,
        timeout: Duration,
    ) -> Result<(), String> {
        // Transport request id only, not the pipeline correlation id
        let request_id = Uuid::now_v7().to_string();

        let mut frame = Map::new();
        frame.insert("type".into(), json!("publish"));
        frame.insert("requestId".into(), json!(request_id));
        frame.insert("topic".into(), json!(request.topic));

        if let Some(key) = &request.key {
            frame.insert("key".into(), json!(key));
        }

        frame.insert("value".into(), request.value);

        if !request.headers.is_empty() {
            frame.insert("headers".into(), json!(request.headers));
        }

        let (sender, receiver) = oneshot::channel();

        self.pending().insert(request_id.clone(), sender);

        tracing::info!(
            target: LOG_TARGET,
            "publish requestId={} topic={} key={}",
            request_id,
            request.topic,
            request.key.as_deref().unwrap_or("")
        );

        // A closed queue means the socket is gone
        if self
            .outbound
            .send(Value::Object(frame).to_string())
            .is_err()
        {
            self.remove_pending(&request_id);

            return Err("WebSocket not open".to_string());
        }

        match tokio::time::timeout(timeout, receiver).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(format!("publish cancelled requestId={request_id}")),
            Err(_) => {
                self.remove_pending(&request_id);

                Err(format!(
                    "publish timeout requestId={request_id} topic={}",
                    request.topic
                ))
            }
        }
    }

    // Handles a published or error answer that carries a request id, true when it was consumed
    pub fn handle_control(&self, message: &Value) -> bool {
        let Some(request_id) = message.get("requestId").and_then(Value::as_str) else {
            return false;
        };

        let Some(sender) = self.remove_pending(request_id) else {
            return false;
        };

        match message.get("type").and_then(Value::as_str) {
            Some("published") => {
                let _ = sender.send(Ok(()));

                true
            }
            Some("error") => {
                let reason = message
                    .get("reason")
                    .and_then(Value::as_str)
                    .or_else(|| message.get("message").and_then(Value::as_str))
                    .unwrap_or("publish error");

                let _ = sender.send(Err(reason.to_string()));

                true
            }
            _ => false,
        }
    }

    // Fails every waiting publish, used when the socket closes
    pub fn reject_all(&self, reason: &str) {
        let drained: Vec<(String, Pending)> = self.pending().drain().collect();

        for (request_id, sender) in drained {
            let _ = sender.send(Err(format!("{reason} requestId={request_id}")));
        }
    }

    // Tells the gateway the event was fully processed so it can commit the offset
    pub fn send_processed(&self, correlation_id: &str) -> Result<(), String> {
        let frame = json!({ "type": "processed", "correlationId": correlation_id });

        self.outbound
            .send(frame.to_string())
            .map_err(|_| "WebSocket not open; cannot send processed ACK".to_string())?;

        tracing::info!(
            target: LOG_TARGET,
            "ACK processed correlationId={correlation_id}"
        );

        Ok(())
    }

    // Locks the waiting publishes, a lock poisoned by a panic is still usable (no unwrap after start-up)
    fn pending(&self) -> MutexGuard<'_, HashMap<String, Pending>> {
        self.pending.lock().unwrap_or_else(PoisonError::into_inner)
    }

    // Removes and returns the waiting publish for a request id
    fn remove_pending(&self, request_id: &str) -> Option<Pending> {
        self.pending().remove(request_id)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    // Builds a transport and the receiving end of its frame queue
    fn transport() -> (Arc<Transport>, mpsc::UnboundedReceiver<String>) {
        let (outbound, queue) = mpsc::unbounded_channel();

        (Arc::new(Transport::new(outbound)), queue)
    }

    // A request with a key and one header
    fn request() -> PublishRequest {
        PublishRequest {
            topic: "topic-1".to_string(),
            key: Some("key-1".to_string()),
            value: json!({ "a": 1 }),
            headers: HashMap::from([("x-jobs-retry-count".to_string(), "1".to_string())]),
        }
    }

    // Reads the next queued frame as json
    async fn next_frame(queue: &mut mpsc::UnboundedReceiver<String>) -> Value {
        let Some(text) = queue.recv().await else {
            panic!("no frame queued");
        };

        let Ok(frame) = serde_json::from_str(&text) else {
            panic!("frame is not json");
        };

        frame
    }

    // A publish frame carries type, request id, topic, key, value and headers, and a published answer resolves it
    #[tokio::test]
    async fn publish_sends_frame_and_resolves_on_published() {
        let (transport, mut queue) = transport();
        let sender = Arc::clone(&transport);

        let task = tokio::spawn(async move { sender.publish(request()).await });

        let frame = next_frame(&mut queue).await;

        assert_eq!(frame["type"], "publish");
        assert_eq!(frame["topic"], "topic-1");
        assert_eq!(frame["key"], "key-1");
        assert_eq!(frame["value"], json!({ "a": 1 }));
        assert_eq!(frame["headers"], json!({ "x-jobs-retry-count": "1" }));

        let consumed = transport.handle_control(&json!({
            "type": "published",
            "requestId": frame["requestId"],
        }));

        assert!(consumed);
        assert_eq!(task.await.ok(), Some(Ok(())));
    }

    // Key and headers are left out when there are none
    #[tokio::test]
    async fn publish_leaves_out_empty_key_and_headers() {
        let (transport, mut queue) = transport();
        let sender = Arc::clone(&transport);

        let bare = PublishRequest {
            topic: "topic-1".to_string(),
            key: None,
            value: json!(1),
            headers: HashMap::new(),
        };

        let task = tokio::spawn(async move { sender.publish(bare).await });

        let frame = next_frame(&mut queue).await;

        assert!(frame.get("key").is_none());
        assert!(frame.get("headers").is_none());

        transport.handle_control(&json!({ "type": "published", "requestId": frame["requestId"] }));

        assert_eq!(task.await.ok(), Some(Ok(())));
    }

    // An error answer resolves the publish with the reason, or the message when there is no reason
    #[tokio::test]
    async fn error_answer_resolves_with_reason() {
        for (answer_key, expected) in [("reason", "no way"), ("message", "no way")] {
            let (transport, mut queue) = transport();
            let sender = Arc::clone(&transport);

            let task = tokio::spawn(async move { sender.publish(request()).await });

            let frame = next_frame(&mut queue).await;

            transport.handle_control(&json!({
                "type": "error",
                "requestId": frame["requestId"],
                answer_key: "no way",
            }));

            assert_eq!(task.await.ok(), Some(Err(expected.to_string())));
        }
    }

    // An answer with an unknown request id is not consumed
    #[test]
    fn unknown_request_id_is_not_consumed() {
        let (transport, _queue) = transport();

        assert!(!transport.handle_control(&json!({ "type": "published", "requestId": "x" })));
        assert!(!transport.handle_control(&json!({ "type": "published" })));
    }

    // A closed queue fails the publish at once
    #[tokio::test]
    async fn closed_queue_fails_the_publish() {
        let (transport, queue) = transport();

        drop(queue);

        let result = transport.publish(request()).await;

        assert_eq!(result, Err("WebSocket not open".to_string()));
    }

    // A publish without an answer times out and its pending entry is removed
    #[tokio::test]
    async fn publish_times_out_without_an_answer() {
        let (transport, _queue) = transport();

        let result = transport
            .publish_with_timeout(request(), Duration::from_millis(30))
            .await;

        let Err(reason) = result else {
            panic!("publish should time out");
        };

        assert!(reason.starts_with("publish timeout requestId="));
        assert!(transport.pending().is_empty());
    }

    // Closing the socket fails every waiting publish
    #[tokio::test]
    async fn reject_all_fails_waiting_publishes() {
        let (transport, mut queue) = transport();
        let sender = Arc::clone(&transport);

        let task = tokio::spawn(async move { sender.publish(request()).await });

        next_frame(&mut queue).await;

        transport.reject_all("WebSocket closed");

        let Ok(Err(reason)) = task.await else {
            panic!("publish should fail");
        };

        assert!(reason.starts_with("WebSocket closed requestId="));
    }

    // The processed acknowledgement frame carries the correlation id
    #[tokio::test]
    async fn send_processed_queues_the_ack_frame() {
        let (transport, mut queue) = transport();

        assert_eq!(transport.send_processed("corr-1"), Ok(()));

        let frame = next_frame(&mut queue).await;

        assert_eq!(
            frame,
            json!({ "type": "processed", "correlationId": "corr-1" })
        );
    }

    // The acknowledgement fails when the socket is gone
    #[test]
    fn send_processed_fails_when_the_queue_is_closed() {
        let (transport, queue) = transport();

        drop(queue);

        assert!(transport.send_processed("corr-1").is_err());
    }
}
