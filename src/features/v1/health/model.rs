use serde::Serialize;
use utoipa::ToSchema;

// Application health data returned inside GET /api/v1/health
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AppHealthData {
    pub status: String,
    pub version: String,
    pub uptime_ms: u64,
    pub timestamp_utc: String,
}

// Overall database status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DbStatus {
    Up,
    Down,
}

// Database connection state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DbConnection {
    Connected,
    Connecting,
    Disconnected,
}

// Database health data shown on the db health pages
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DbHealthData {
    pub status: DbStatus,
    pub database: DbConnection,
    pub latency_ms: Option<u64>,
}

// Combined app and database health data returned by GET /api/v1/health and GET /healthz
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthData {
    pub app: AppHealthData,
    pub db: DbHealthData,
}
