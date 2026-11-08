use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::HeaderMap,
    response::Response,
    routing::get,
};
use serde_json::{Value, json};
use subscriber::{
    gateway::{self, Backoff, GatewayConfig, GatewayEvent},
    transport::PublishRequest,
};
use tokio::sync::mpsc;

// Short reconnect delays so the tests stay fast
const BACKOFF: Backoff = Backoff {
    base: Duration::from_millis(10),
    max: Duration::from_millis(50),
};

// State of the fake gateway: what it sends and what it saw
#[derive(Clone)]
struct Fake {
    connections: Arc<AtomicUsize>,
    headers: Arc<Mutex<Vec<HeaderMap>>>,
    frames: Arc<Mutex<Vec<Value>>>,
    events: Arc<Vec<Value>>,
    drop_first: bool,
    fail_publish: bool,
}

impl Fake {
    // Builds a fake that sends the events after the third subscribe frame
    fn new(events: Vec<Value>, drop_first: bool, fail_publish: bool) -> Self {
        Self {
            connections: Arc::new(AtomicUsize::new(0)),
            headers: Arc::new(Mutex::new(Vec::new())),
            frames: Arc::new(Mutex::new(Vec::new())),
            events: Arc::new(events),
            drop_first,
            fail_publish,
        }
    }

    // Frames of the client with the given type
    fn frames_of(&self, kind: &str) -> Vec<Value> {
        self.frames
            .lock()
            .unwrap()
            .iter()
            .filter(|frame| frame["type"] == kind)
            .cloned()
            .collect()
    }
}

// Fake subscribe route, records the handshake headers and starts a session
async fn ws_route(
    State(fake): State<Fake>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let index = fake.connections.fetch_add(1, Ordering::SeqCst);

    fake.headers.lock().unwrap().push(headers);

    upgrade.on_upgrade(move |socket| session(socket, fake, index))
}

// One fake gateway session: ready, then answers subscribe and publish frames
async fn session(mut socket: WebSocket, fake: Fake, index: usize) {
    if socket
        .send(Message::text(json!({ "type": "ready" }).to_string()))
        .await
        .is_err()
    {
        return;
    }

    let mut subscribes = 0;

    while let Some(Ok(message)) = socket.recv().await {
        let Message::Text(text) = message else {
            continue;
        };

        let Ok(frame) = serde_json::from_str::<Value>(text.as_str()) else {
            continue;
        };

        fake.frames.lock().unwrap().push(frame.clone());

        match frame["type"].as_str() {
            Some("subscribe") => {
                subscribes += 1;

                if subscribes == 3 {
                    // The first connection is closed on purpose to test the reconnect
                    if fake.drop_first && index == 0 {
                        return;
                    }

                    for event in fake.events.iter() {
                        if socket.send(Message::text(event.to_string())).await.is_err() {
                            return;
                        }
                    }
                }
            }
            Some("publish") => {
                let reply = if fake.fail_publish {
                    json!({ "type": "error", "requestId": frame["requestId"], "reason": "gateway said no" })
                } else {
                    json!({ "type": "published", "requestId": frame["requestId"] })
                };

                if socket.send(Message::text(reply.to_string())).await.is_err() {
                    return;
                }
            }
            _ => {}
        }
    }
}

// Starts the fake gateway on a free local port and returns the config that points at it
async fn start(fake: Fake) -> GatewayConfig {
    let router = Router::new()
        .route("/api/v1/ws-subscribe", get(ws_route))
        .with_state(fake);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    GatewayConfig {
        url: format!("ws://{address}/api/v1/ws-subscribe"),
        client_id: "client-1".to_string(),
        client_secret: "secret-1".to_string(),
        api_key: "key-1".to_string(),
        api_secret: "key-secret-1".to_string(),
        subscriber_id: "source-1".to_string(),
    }
}

// The three topics the tests subscribe to
fn topics() -> Vec<String> {
    vec![
        "t-write".to_string(),
        "t-read".to_string(),
        "t-status".to_string(),
    ]
}

// Waits until the condition holds, fails the test after 5 seconds
async fn wait_until(mut condition: impl FnMut() -> bool) {
    for _ in 0..250 {
        if condition() {
            return;
        }

        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    panic!("condition not met in time");
}

// Starts the subscriber against the fake and returns the events its handler receives
fn run_with_event_channel(
    config: GatewayConfig,
) -> (
    tokio::task::JoinHandle<()>,
    mpsc::UnboundedReceiver<GatewayEvent>,
) {
    let (event_tx, event_rx) = mpsc::unbounded_channel::<GatewayEvent>();

    let task = tokio::spawn(gateway::run(
        config,
        topics(),
        move |event| {
            let event_tx = event_tx.clone();

            async move {
                let _ = event_tx.send(event);
            }
        },
        BACKOFF,
    ));

    (task, event_rx)
}

// An event frame the fake sends
fn event_frame() -> Value {
    json!({
        "type": "event",
        "topic": "t-write",
        "headers": { "x-jobs-retry-count": "2", "number": 5 },
        "partition": 3,
        "offset": 42,
        "correlationId": "c1",
        "job": { "writeModel": "w", "readModel": "r" },
    })
}

// Receives the next event or fails the test after 5 seconds
async fn next_event(events: &mut mpsc::UnboundedReceiver<GatewayEvent>) -> GatewayEvent {
    match tokio::time::timeout(Duration::from_secs(5), events.recv()).await {
        Ok(Some(event)) => event,
        _ => panic!("no event received"),
    }
}

// The handshake carries the five headers and the subscriber subscribes to all three topics
#[tokio::test]
async fn connects_with_headers_and_subscribes_to_three_topics() {
    let fake = Fake::new(Vec::new(), false, false);
    let config = start(fake.clone()).await;
    let (task, _events) = run_with_event_channel(config);

    wait_until(|| fake.frames_of("subscribe").len() == 3).await;

    let topics: Vec<String> = fake
        .frames_of("subscribe")
        .iter()
        .map(|frame| frame["topic"].as_str().unwrap_or_default().to_string())
        .collect();

    assert_eq!(topics, ["t-write", "t-read", "t-status"]);

    let headers = fake.headers.lock().unwrap()[0].clone();

    for (name, value) in [
        ("x-client-id", "client-1"),
        ("x-client-secret", "secret-1"),
        ("x-api-key", "key-1"),
        ("x-api-secret", "key-secret-1"),
        ("x-subscriber-id", "source-1"),
    ] {
        assert_eq!(headers[name], value, "header {name}");
    }

    task.abort();
}

// An event frame reaches the handler split into topic, payload, headers, partition and offset
#[tokio::test]
async fn delivers_events_to_the_handler() {
    let fake = Fake::new(vec![event_frame()], false, false);
    let config = start(fake).await;
    let (task, mut events) = run_with_event_channel(config);

    let event = next_event(&mut events).await;

    assert_eq!(event.topic, "t-write");
    assert_eq!(
        event.payload,
        json!({ "correlationId": "c1", "job": { "writeModel": "w", "readModel": "r" } })
    );
    assert_eq!(
        event.meta.headers,
        HashMap::from([("x-jobs-retry-count".to_string(), "2".to_string())])
    );
    assert_eq!(event.meta.partition, Some(3));
    assert_eq!(event.meta.offset.as_deref(), Some("42"));

    task.abort();
}

// A publish through the event transport is confirmed by the gateway and carries key and headers
#[tokio::test]
async fn publish_is_confirmed_by_the_gateway() {
    let fake = Fake::new(vec![event_frame()], false, false);
    let config = start(fake.clone()).await;
    let (task, mut events) = run_with_event_channel(config);

    let event = next_event(&mut events).await;

    let result = event
        .transport
        .publish(PublishRequest {
            topic: "t-status".to_string(),
            key: Some("c1".to_string()),
            value: json!({ "a": 1 }),
            headers: HashMap::from([("x-jobs-retry-count".to_string(), "1".to_string())]),
        })
        .await;

    assert_eq!(result, Ok(()));

    let frames = fake.frames_of("publish");

    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0]["topic"], "t-status");
    assert_eq!(frames[0]["key"], "c1");
    assert_eq!(frames[0]["value"], json!({ "a": 1 }));
    assert_eq!(frames[0]["headers"], json!({ "x-jobs-retry-count": "1" }));

    task.abort();
}

// An error answer from the gateway fails the publish with its reason
#[tokio::test]
async fn publish_error_carries_the_gateway_reason() {
    let fake = Fake::new(vec![event_frame()], false, true);
    let config = start(fake).await;
    let (task, mut events) = run_with_event_channel(config);

    let event = next_event(&mut events).await;

    let result = event
        .transport
        .publish(PublishRequest {
            topic: "t-status".to_string(),
            key: None,
            value: json!(1),
            headers: HashMap::new(),
        })
        .await;

    assert_eq!(result, Err("gateway said no".to_string()));

    task.abort();
}

// The processed acknowledgement reaches the gateway
#[tokio::test]
async fn processed_ack_reaches_the_gateway() {
    let fake = Fake::new(vec![event_frame()], false, false);
    let config = start(fake.clone()).await;
    let (task, mut events) = run_with_event_channel(config);

    let event = next_event(&mut events).await;

    assert_eq!(event.transport.send_processed("c1"), Ok(()));

    wait_until(|| !fake.frames_of("processed").is_empty()).await;

    assert_eq!(fake.frames_of("processed")[0]["correlationId"], "c1");

    task.abort();
}

// After the gateway closes the connection the subscriber reconnects and subscribes again
#[tokio::test]
async fn reconnects_after_the_gateway_closes_the_connection() {
    let fake = Fake::new(vec![event_frame()], true, false);
    let config = start(fake.clone()).await;
    let (task, mut events) = run_with_event_channel(config);

    // The event only comes on the second connection
    let event = next_event(&mut events).await;

    assert_eq!(event.topic, "t-write");
    assert!(fake.connections.load(Ordering::SeqCst) >= 2);
    assert_eq!(fake.frames_of("subscribe").len(), 6);

    task.abort();
}
