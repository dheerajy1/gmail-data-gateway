use std::collections::HashMap;

// Maximum retry count stored on a message, the first delivery is 0 (SPEC section 8.3)
pub const MAX_RETRIES: u32 = 3;

// Kafka header with the current retry attempt, jobs keeps its own name (SPEC section 8.3)
pub const RETRY_COUNT_HEADER: &str = "x-jobs-retry-count";

// Kafka headers added to a message that goes to a dead letter topic
pub const HEADER_ORIGINAL_TOPIC: &str = "x-original-topic";
pub const HEADER_ORIGINAL_PARTITION: &str = "x-original-partition";
pub const HEADER_ORIGINAL_OFFSET: &str = "x-original-offset";
pub const HEADER_CORRELATION_ID: &str = "x-correlation-id";
pub const HEADER_FAILURE_REASON: &str = "x-failure-reason";

// Reads the retry count from the headers, anything missing or invalid counts as 0
pub fn get_retry_count(headers: &HashMap<String, String>) -> u32 {
    headers
        .get(RETRY_COUNT_HEADER)
        .and_then(|raw| raw.parse::<u32>().ok())
        .unwrap_or(0)
}

// True when the message has used up its retries and must go to the dead letter topic
pub fn should_dlq(current_retry_count: u32) -> bool {
    current_retry_count >= MAX_RETRIES
}

#[cfg(test)]
mod tests {
    use super::*;

    // Builds headers with a single retry count value
    fn headers_with(value: &str) -> HashMap<String, String> {
        HashMap::from([(RETRY_COUNT_HEADER.to_string(), value.to_string())])
    }

    // A message without the header is on its first delivery
    #[test]
    fn missing_header_counts_as_zero() {
        assert_eq!(get_retry_count(&HashMap::new()), 0);
    }

    // A valid header value is read as the retry count
    #[test]
    fn reads_retry_count_from_header() {
        assert_eq!(get_retry_count(&headers_with("2")), 2);
    }

    // A header value that is not a number counts as 0
    #[test]
    fn invalid_header_counts_as_zero() {
        assert_eq!(get_retry_count(&headers_with("abc")), 0);
        assert_eq!(get_retry_count(&headers_with("-1")), 0);
        assert_eq!(get_retry_count(&headers_with("")), 0);
    }

    // The recordlog header name is not read by jobs
    #[test]
    fn recordlog_header_is_ignored() {
        let headers = HashMap::from([("x-record-log-retry-count".to_string(), "2".to_string())]);

        assert_eq!(get_retry_count(&headers), 0);
    }

    // Messages below the maximum are retried
    #[test]
    fn retries_below_maximum() {
        assert!(!should_dlq(0));
        assert!(!should_dlq(MAX_RETRIES - 1));
    }

    // Messages at or above the maximum go to the dead letter topic
    #[test]
    fn dead_letters_at_maximum() {
        assert!(should_dlq(MAX_RETRIES));
        assert!(should_dlq(MAX_RETRIES + 1));
    }
}
