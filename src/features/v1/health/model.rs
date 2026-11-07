use serde::Serialize;

// Application health data returned by GET /api/v1/health
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppHealthData {
    pub status: String,
    pub version: String,
    pub uptime_ms: u64,
    pub timestamp_utc: String,
}

// Health data of the whole service, the database part is added in P3
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthData {
    pub app: AppHealthData,
}
