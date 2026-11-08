use chrono::{FixedOffset, NaiveDate, TimeZone};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::errors::{api_error::ApiError, catalog::bad_request};

// India time is a fixed +05:30 offset, a date-only input means midnight in that offset (D47)
pub(super) const IST_OFFSET_SECONDS: i32 = 19_800;

// The three extract modes (SPEC section 5.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
pub enum ExtractType {
    Bulk,
    Increment,
    Sync,
}

impl ExtractType {
    // Name of the mode as the client sends it
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bulk => "Bulk",
            Self::Increment => "Increment",
            Self::Sync => "Sync",
        }
    }
}

// Body of POST /api/v1/admin/extractMails (SPEC section 5.1)
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExtractMailsRequest {
    // Extract mode
    #[serde(rename = "type")]
    pub extract_type: ExtractType,

    // Gmail search text, at least 1 character
    pub query: String,

    // Hard maximum of mails, ignored in Bulk mode
    pub input_mails_count: Option<u32>,

    // Start date as YYYY/M/D, Increment mode only
    pub input_date_after: Option<String>,

    // End date as YYYY/M/D, Increment mode only
    pub input_date_before: Option<String>,
}

// Data part of the extractMails success response (SPEC section 5.6)
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExtractMailsData {
    // Correlation id of each published chunk, oldest chunk first
    pub correlation_ids: Vec<String>,

    // Total records across all chunks
    pub record_count: usize,

    // Number of chunks published
    pub chunk_count: usize,

    // ACCEPTED, or NO_NEW_RECORDS when nothing was found
    pub status: String,
}

// True when the text is only ascii digits
fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|character| character.is_ascii_digit())
}

// Turns a YYYY/M/D date into unix seconds at midnight India time, None for any other text
pub fn ist_midnight_unix(text: &str) -> Option<i64> {
    let mut parts = text.split('/');

    let (Some(year), Some(month), Some(day), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };

    if year.len() != 4 || !is_digits(year) || !is_digits(month) || !is_digits(day) {
        return None;
    }

    let date = NaiveDate::from_ymd_opt(year.parse().ok()?, month.parse().ok()?, day.parse().ok()?)?;

    let offset = FixedOffset::east_opt(IST_OFFSET_SECONDS)?;

    offset
        .from_local_datetime(&date.and_hms_opt(0, 0, 0)?)
        .single()
        .map(|midnight| midnight.timestamp())
}

impl ExtractMailsRequest {
    // Checks the field and mode rules of SPEC section 5.1, any failure is 400 INVALID_INPUT
    pub fn validate(&self) -> Result<(), ApiError> {
        let invalid = || ApiError::from_entry(bad_request::INVALID_INPUT);

        if self.query.is_empty() || self.input_mails_count == Some(0) {
            return Err(invalid());
        }

        let has_after = self.input_date_after.is_some();
        let has_before = self.input_date_before.is_some();

        match self.extract_type {
            // Bulk and Sync take no date field, Sync reads its date from the database
            ExtractType::Bulk | ExtractType::Sync if has_after || has_before => {
                return Err(invalid());
            }
            // Increment needs at least one date field
            ExtractType::Increment if !has_after && !has_before => return Err(invalid()),
            _ => {}
        }

        let dates = [&self.input_date_after, &self.input_date_before];

        if dates
            .into_iter()
            .flatten()
            .any(|date| ist_midnight_unix(date).is_none())
        {
            return Err(invalid());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Builds a request with the given mode, query and optional fields
    fn request(
        extract_type: ExtractType,
        query: &str,
        count: Option<u32>,
        after: Option<&str>,
        before: Option<&str>,
    ) -> ExtractMailsRequest {
        ExtractMailsRequest {
            extract_type,
            query: query.to_string(),
            input_mails_count: count,
            input_date_after: after.map(str::to_string),
            input_date_before: before.map(str::to_string),
        }
    }

    // Dates without padding and with padding both become midnight India time
    #[test]
    fn dates_become_ist_midnight() {
        // 2026-11-02 00:00 +05:30 is 2026-11-01 18:30 UTC
        assert_eq!(ist_midnight_unix("2026/11/2"), Some(1_793_557_800));
        assert_eq!(ist_midnight_unix("2026/11/02"), Some(1_793_557_800));
        assert_eq!(ist_midnight_unix("2026/11/5"), Some(1_793_817_000));
    }

    // Other formats and impossible dates do not parse
    #[test]
    fn bad_dates_do_not_parse() {
        for text in [
            "",
            "2026-11-02",
            "2026/11",
            "2026/11/2/1",
            "2026/13/1",
            "2026/2/30",
            "26/11/2",
            "+2026/11/2",
            "2026/ 1/2",
            "2026//2",
            "abcd/11/2",
        ] {
            assert_eq!(ist_midnight_unix(text), None, "{text}");
        }
    }

    // Bulk and Sync with a query and no dates are valid
    #[test]
    fn bulk_and_sync_without_dates_are_valid() {
        assert!(
            request(ExtractType::Bulk, "jobs", None, None, None)
                .validate()
                .is_ok()
        );
        assert!(
            request(ExtractType::Sync, "jobs", Some(10), None, None)
                .validate()
                .is_ok()
        );
    }

    // Bulk and Sync reject any date field
    #[test]
    fn bulk_and_sync_reject_dates() {
        for extract_type in [ExtractType::Bulk, ExtractType::Sync] {
            assert!(
                request(extract_type, "jobs", None, Some("2026/11/2"), None)
                    .validate()
                    .is_err()
            );
            assert!(
                request(extract_type, "jobs", None, None, Some("2026/11/5"))
                    .validate()
                    .is_err()
            );
        }
    }

    // Increment needs at least one date and accepts one or both
    #[test]
    fn increment_needs_a_date() {
        assert!(
            request(ExtractType::Increment, "jobs", None, None, None)
                .validate()
                .is_err()
        );
        assert!(
            request(
                ExtractType::Increment,
                "jobs",
                None,
                Some("2026/11/2"),
                None
            )
            .validate()
            .is_ok()
        );
        assert!(
            request(
                ExtractType::Increment,
                "jobs",
                None,
                None,
                Some("2026/11/5")
            )
            .validate()
            .is_ok()
        );
        assert!(
            request(
                ExtractType::Increment,
                "jobs",
                None,
                Some("2026/11/2"),
                Some("2026/11/5")
            )
            .validate()
            .is_ok()
        );
    }

    // An Increment with an unparsable date is rejected
    #[test]
    fn increment_rejects_bad_dates() {
        assert!(
            request(
                ExtractType::Increment,
                "jobs",
                None,
                Some("11/2/2026"),
                None
            )
            .validate()
            .is_err()
        );
    }

    // An empty query and a zero count are rejected
    #[test]
    fn empty_query_and_zero_count_are_rejected() {
        assert!(
            request(ExtractType::Bulk, "", None, None, None)
                .validate()
                .is_err()
        );
        assert!(
            request(ExtractType::Sync, "jobs", Some(0), None, None)
                .validate()
                .is_err()
        );
    }

    // The body json uses the camelCase names and the exact mode names
    #[test]
    fn body_json_parses() {
        let body = r#"{"type":"Increment","query":"jobs","inputMailsCount":5,"inputDateAfter":"2026/11/2","inputDateBefore":"2026/11/5"}"#;
        let parsed: ExtractMailsRequest = serde_json::from_str(body).unwrap();

        assert_eq!(parsed.extract_type, ExtractType::Increment);
        assert_eq!(parsed.input_mails_count, Some(5));
        assert_eq!(parsed.input_date_after.as_deref(), Some("2026/11/2"));
    }

    // An unknown mode or a missing query does not parse
    #[test]
    fn bad_bodies_do_not_parse() {
        assert!(
            serde_json::from_str::<ExtractMailsRequest>(r#"{"type":"bulk","query":"jobs"}"#)
                .is_err()
        );
        assert!(serde_json::from_str::<ExtractMailsRequest>(r#"{"type":"Bulk"}"#).is_err());
    }
}
