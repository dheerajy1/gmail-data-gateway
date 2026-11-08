use super::model::{ExtractMailsRequest, ExtractType, ist_midnight_unix};
use crate::errors::{
    api_error::ApiError,
    catalog::{bad_request, internal_server_error},
};

// Gmail search text and the mail limit for one extract run (SPEC section 5.2)
#[derive(Debug, PartialEq, Eq)]
pub struct BuiltQuery {
    // Final text sent to Gmail as the q parameter
    pub q: String,

    // Hard maximum of mails, None means all pages
    pub limit: Option<u32>,
}

// Builds the Gmail query for a request, last_stored_unix is only used by Sync
pub fn build_query(
    request: &ExtractMailsRequest,
    last_stored_unix: Option<i64>,
) -> Result<BuiltQuery, ApiError> {
    match request.extract_type {
        // Bulk reads every page and ignores the count
        ExtractType::Bulk => Ok(BuiltQuery {
            q: request.query.clone(),
            limit: None,
        }),
        // Sync continues after the last stored date
        ExtractType::Sync => {
            let after = last_stored_unix
                .ok_or_else(|| ApiError::from_entry(internal_server_error::EXTRACT_FAILED))?;

            Ok(BuiltQuery {
                q: format!("{} after:{after}", request.query),
                limit: request.input_mails_count,
            })
        }
        // Increment uses the dates as given, after first and then before
        ExtractType::Increment => {
            let mut q = request.query.clone();

            for (name, date) in [
                ("after", &request.input_date_after),
                ("before", &request.input_date_before),
            ] {
                if let Some(date) = date {
                    let unix = ist_midnight_unix(date)
                        .ok_or_else(|| ApiError::from_entry(bad_request::INVALID_INPUT))?;

                    q.push_str(&format!(" {name}:{unix}"));
                }
            }

            Ok(BuiltQuery {
                q,
                limit: request.input_mails_count,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Builds a request with the given mode and optional fields, the query is always "jobs"
    fn request(
        extract_type: ExtractType,
        count: Option<u32>,
        after: Option<&str>,
        before: Option<&str>,
    ) -> ExtractMailsRequest {
        ExtractMailsRequest {
            extract_type,
            query: "jobs".to_string(),
            input_mails_count: count,
            input_date_after: after.map(str::to_string),
            input_date_before: before.map(str::to_string),
        }
    }

    // Bulk sends the query as it is and has no limit, even with a count
    #[test]
    fn bulk_has_no_limit() {
        let built = build_query(&request(ExtractType::Bulk, Some(5), None, None), None).unwrap();

        assert_eq!(
            built,
            BuiltQuery {
                q: "jobs".to_string(),
                limit: None
            }
        );
    }

    // Sync adds after with the last stored date and keeps the count as the limit
    #[test]
    fn sync_adds_after_from_the_database() {
        let built = build_query(
            &request(ExtractType::Sync, Some(10), None, None),
            Some(1_787_118_907),
        )
        .unwrap();

        assert_eq!(built.q, "jobs after:1787118907");
        assert_eq!(built.limit, Some(10));
    }

    // Sync without a stored date is an extract failure
    #[test]
    fn sync_without_a_stored_date_fails() {
        assert!(build_query(&request(ExtractType::Sync, None, None, None), None).is_err());
    }

    // Increment with both dates adds after and then before
    #[test]
    fn increment_adds_after_and_before() {
        let built = build_query(
            &request(
                ExtractType::Increment,
                None,
                Some("2026/11/2"),
                Some("2026/11/5"),
            ),
            None,
        )
        .unwrap();

        assert_eq!(built.q, "jobs after:1793557800 before:1793817000");
        assert_eq!(built.limit, None);
    }

    // Increment with one date adds only that one
    #[test]
    fn increment_with_one_date_adds_one() {
        let only_after = build_query(
            &request(ExtractType::Increment, Some(3), Some("2026/11/2"), None),
            None,
        )
        .unwrap();
        let only_before = build_query(
            &request(ExtractType::Increment, None, None, Some("2026/11/5")),
            None,
        )
        .unwrap();

        assert_eq!(only_after.q, "jobs after:1793557800");
        assert_eq!(only_after.limit, Some(3));
        assert_eq!(only_before.q, "jobs before:1793817000");
    }
}
