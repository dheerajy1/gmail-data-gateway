use chrono::DateTime;
use tiberius::error::Error as SqlError;

use crate::{
    db::azure::service_pool,
    errors::{
        api_error::ApiError,
        catalog::{internal_server_error, not_found},
    },
};

// Error number the procedure raises when the table has no rows
const NO_ROWS_ERROR: u32 = 50001;

// Text format of the date the procedure returns, for example 2026-08-19 11:25:07.0000000 +05:30
const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.f %:z";

// Turns the procedure date text into unix seconds, None when the text has another format
fn parse_last_date(text: &str) -> Option<i64> {
    DateTime::parse_from_str(text, DATE_FORMAT)
        .ok()
        .map(|date| date.timestamp())
}

// Maps a database error of the procedure, an empty table is 404 and everything else is EXTRACT_FAILED
fn map_error(error: SqlError) -> ApiError {
    match &error {
        SqlError::Server(token) if token.code() == NO_ROWS_ERROR => {
            ApiError::from_entry(not_found::NO_EMAIL_DATA)
        }
        _ => {
            tracing::error!(target: "Db", "last applied date query failed: {error}");

            ApiError::from_entry(internal_server_error::EXTRACT_FAILED)
        }
    }
}

// Reads the latest stored applied date as unix seconds from [03spgetLastDateAppliedJobs]
pub async fn get_last_applied_unix() -> Result<i64, ApiError> {
    let mut connection = service_pool().get().await.map_err(|error| {
        tracing::error!(target: "Db", "service pool failed: {error}");

        ApiError::from_entry(internal_server_error::EXTRACT_FAILED)
    })?;

    let row = connection
        .simple_query("EXEC [02sjobsData].[03spgetLastDateAppliedJobs]")
        .await
        .map_err(map_error)?
        .into_row()
        .await
        .map_err(map_error)?;

    let text = row.as_ref().and_then(|row| row.get::<&str, _>("date"));

    let Some(unix) = text.and_then(parse_last_date) else {
        tracing::error!(target: "Db", "last applied date has an unexpected format");

        return Err(ApiError::from_entry(internal_server_error::EXTRACT_FAILED));
    };

    Ok(unix)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The real value returned by your database parses to the right instant
    #[test]
    fn real_database_value_parses() {
        // 2026-08-19 11:25:07 +05:30 is 2026-08-19 05:55:07 UTC
        assert_eq!(
            parse_last_date("2026-08-19 11:25:07.0000000 +05:30"),
            Some(1_787_118_907)
        );
    }

    // A value with another offset is converted to the same instant
    #[test]
    fn other_offset_parses() {
        assert_eq!(
            parse_last_date("2026-08-19 05:55:07.0000000 +00:00"),
            Some(1_787_118_907)
        );
    }

    // Text in another format does not parse
    #[test]
    fn other_formats_do_not_parse() {
        assert_eq!(parse_last_date("2026-08-19T11:25:07+05:30"), None);
        assert_eq!(parse_last_date("not a date"), None);
        assert_eq!(parse_last_date(""), None);
    }
}
