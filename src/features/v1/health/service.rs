use std::sync::LazyLock;
use std::time::{Duration, Instant};

use chrono::{SecondsFormat, Utc};
use tokio::time::timeout;

use super::model::{AppHealthData, DbConnection, DbHealthData, DbStatus};
use crate::db::azure::public_pool;

// Longest the database probe may take before the database counts as down
const DB_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

// Moment the process started, recorded on first use
static STARTED_AT: LazyLock<Instant> = LazyLock::new(Instant::now);

// Forces the start time to be recorded at boot instead of at the first request
pub fn mark_started() {
    LazyLock::force(&STARTED_AT);
}

// Builds the application health data from process state
pub fn get_app_health() -> AppHealthData {
    AppHealthData {
        status: "UP".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_ms: STARTED_AT.elapsed().as_millis() as u64,
        timestamp_utc: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
    }
}

// Runs SELECT 1 on the public pool, true when it succeeds within the timeout
async fn probe_db() -> bool {
    let run = async {
        let mut connection = public_pool().get().await.ok()?;
        let stream = connection.simple_query("SELECT 1").await.ok()?;

        stream.into_results().await.ok()?;

        Some(())
    };

    matches!(timeout(DB_PROBE_TIMEOUT, run).await, Ok(Some(())))
}

// Builds the database health data from a live probe
pub async fn get_db_health() -> DbHealthData {
    let start = Instant::now();

    if probe_db().await {
        DbHealthData {
            status: DbStatus::Up,
            database: DbConnection::Connected,
            latency_ms: Some(start.elapsed().as_millis() as u64),
        }
    } else {
        DbHealthData {
            status: DbStatus::Down,
            database: DbConnection::Disconnected,
            latency_ms: None,
        }
    }
}
