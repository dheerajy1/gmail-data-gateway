use std::{sync::LazyLock, time::Duration};

use jobs_api_gateway::config::env::env;
use serde_json::Value;
use sqlx::{
    AssertSqlSafe, PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};

use crate::event::{AzureWriteRange, JobRecord};

// Most connections the pool opens
const MAX_CONNECTIONS: u32 = 5;

// A connection that stays unused this long is closed
const IDLE_TIMEOUT: Duration = Duration::from_secs(120);

// Pool of on-prem PostgreSQL connections for the service user, no connection is opened until first use
// The server is reached without TLS like the kafka gateway
static POOL: LazyLock<PgPool> = LazyLock::new(|| {
    let env = env();

    let options = PgConnectOptions::new()
        .host(&env.onprem_db_server)
        .port(env.onprem_db_port)
        .database(&env.onprem_db_database)
        .username(&env.onprem_db_user_service)
        .password(&env.onprem_db_password_service)
        .application_name("jobs-api-gateway")
        .ssl_mode(PgSslMode::Disable);

    PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .idle_timeout(Some(IDLE_TIMEOUT))
        .connect_lazy_with(options)
});

// Returns the service user pool, created on first use
fn pool() -> &'static PgPool {
    &POOL
}

// What the read-model procedure reports (SPEC section 9.2)
#[derive(Debug, Clone)]
pub struct ReadModelResult {
    // True when the same correlation id was already applied
    pub replayed: bool,

    // Number of rows inserted
    pub rows_inserted: Option<i32>,
}

// What the apply-status function reports (SPEC section 9.2)
#[derive(Debug, Clone)]
pub struct ApplyStatusResult {
    // True when the same event id was already applied
    pub replayed: bool,

    // Sequence number the database assigned
    pub assigned_sequence: i32,
}

// The procedure name goes into the SQL text, so only safe characters are allowed (SPEC section 9.2)
fn assert_procedure_name(name: &str) -> Result<&str, String> {
    let valid = !name.is_empty()
        && name.len() <= 200
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '"'));

    if valid {
        Ok(name)
    } else {
        Err("Invalid read-model procedure name".to_string())
    }
}

// Runs the read-model procedure inside one transaction, the name comes from the event
pub async fn execute_read_model(
    procedure_name: &str,
    correlation_id: &str,
    records: &[JobRecord],
    azure_write: Option<AzureWriteRange>,
) -> Result<ReadModelResult, String> {
    let procedure = assert_procedure_name(procedure_name)?;
    let records_json = serde_json::to_string(records).map_err(|error| error.to_string())?;

    let mut transaction = pool().begin().await.map_err(|error| error.to_string())?;

    // The procedure name passed assert_procedure_name, so the SQL text is safe (sqlx 0.9 needs that said)
    // The casts go through text so plain strings can be bound, the three NULLs are the OUT parameters
    // The Azure id range is NULL when the write stage gave none, then azure_id stays NULL until the Azure sync
    let row = sqlx::query(AssertSqlSafe(format!(
        "CALL {procedure}($1::text::uuid, $2::text::jsonb, $3::int, $4::int, NULL, NULL, NULL)"
    )))
    .bind(correlation_id)
    .bind(records_json)
    .bind(azure_write.map(|range| range.from_id))
    .bind(azure_write.map(|range| range.to_id))
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    transaction
        .commit()
        .await
        .map_err(|error| error.to_string())?;

    let result: Option<String> = row.try_get("result").map_err(|error| error.to_string())?;

    if result.as_deref() != Some("SUCCESS") {
        return Err("Read-model procedure did not return SUCCESS".to_string());
    }

    Ok(ReadModelResult {
        replayed: row
            .try_get::<Option<bool>, _>("replayed")
            .map_err(|error| error.to_string())?
            .unwrap_or(false),
        rows_inserted: row
            .try_get("rows_inserted")
            .map_err(|error| error.to_string())?,
    })
}

// Saves one pipeline status event, the event id is the idempotency key and is never regenerated
pub async fn apply_status(
    event_id: &str,
    correlation_id: &str,
    stage: &str,
    status: &str,
    timestamp: &str,
    details: Option<&Value>,
) -> Result<ApplyStatusResult, String> {
    let mut transaction = pool().begin().await.map_err(|error| error.to_string())?;

    // The casts go through text so plain strings can be bound
    let row = sqlx::query(
        "SELECT result, replayed, assigned_sequence, current_stage, current_status
           FROM fn_pipeline_apply_status(
             $1::text::uuid,
             $2::text::uuid,
             $3::varchar,
             $4::varchar,
             $5::text::timestamptz,
             $6::text::jsonb
           )",
    )
    .bind(event_id)
    .bind(correlation_id)
    .bind(stage)
    .bind(status)
    .bind(timestamp)
    .bind(details.map(Value::to_string))
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| error.to_string())?;

    transaction
        .commit()
        .await
        .map_err(|error| error.to_string())?;

    let result: Option<String> = row.try_get("result").map_err(|error| error.to_string())?;

    if result.as_deref() != Some("SUCCESS") {
        return Err("fn_pipeline_apply_status did not return SUCCESS".to_string());
    }

    Ok(ApplyStatusResult {
        replayed: row
            .try_get::<Option<bool>, _>("replayed")
            .map_err(|error| error.to_string())?
            .unwrap_or(false),
        assigned_sequence: row
            .try_get::<Option<i32>, _>("assigned_sequence")
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Plain, schema qualified and quoted procedure names are accepted
    #[test]
    fn accepts_safe_procedure_names() {
        assert_eq!(
            assert_procedure_name("sp_kafka_apply_committed_records"),
            Ok("sp_kafka_apply_committed_records")
        );
        assert_eq!(
            assert_procedure_name("public.sp_kafka_apply_committed_records"),
            Ok("public.sp_kafka_apply_committed_records")
        );
        assert_eq!(
            assert_procedure_name("\"public\".\"sp_apply\""),
            Ok("\"public\".\"sp_apply\"")
        );
    }

    // An empty name is rejected
    #[test]
    fn rejects_empty_procedure_name() {
        assert!(assert_procedure_name("").is_err());
    }

    // Names with characters that could change the SQL text are rejected, square brackets are Azure only
    #[test]
    fn rejects_unsafe_procedure_names() {
        assert!(assert_procedure_name("proc; DROP TABLE jobs").is_err());
        assert!(assert_procedure_name("proc--comment").is_err());
        assert!(assert_procedure_name("proc'name").is_err());
        assert!(assert_procedure_name("proc name").is_err());
        assert!(assert_procedure_name("[dbo].[proc]").is_err());
    }

    // A name over 200 characters is rejected
    #[test]
    fn rejects_too_long_procedure_name() {
        assert!(assert_procedure_name(&"a".repeat(201)).is_err());
        assert!(assert_procedure_name(&"a".repeat(200)).is_ok());
    }
}
