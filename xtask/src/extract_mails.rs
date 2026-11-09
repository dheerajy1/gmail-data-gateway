use std::process::exit;

use jobs_api_gateway::{
    config::env,
    features::v1::extract_mails::{
        model::{ExtractMailsRequest, ExtractType},
        service::extract_mails,
    },
    logging,
};

// How the command is typed, shown when an argument is wrong
const USAGE: &str = "Usage: cargo extract-mails <Bulk|Increment|Sync> \"<gmail query>\" [--count N] [--after YYYY/M/D] [--before YYYY/M/D]";

// Runs one extract from the terminal, without the HTTP route, the client headers or the JWT
// It reads Gmail and publishes to the gateway exactly like POST /api/v1/admin/extractMails
// The env file must already be loaded by the caller
pub fn run(args: &[String]) {
    let request = parse_args(args).unwrap_or_else(|error| {
        eprintln!("{error}\n{USAGE}");
        exit(1);
    });

    logging::init();
    env::init();

    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("[extractMails:Error]\t Failed to start the runtime: {error}");
            exit(1);
        }
    };

    match runtime.block_on(extract_mails(&request)) {
        Ok(outcome) => {
            println!("{}", outcome.message);
            println!("status: {}", outcome.data.status);
            println!("recordCount: {}", outcome.data.record_count);
            println!("chunkCount: {}", outcome.data.chunk_count);

            for correlation_id in outcome.data.correlation_ids {
                println!("correlationId: {correlation_id}");
            }
        }
        Err(error) => {
            eprintln!("[extractMails:Error]\t {error}");
            exit(1);
        }
    }
}

// Turns the typed arguments into the request body of SPEC section 5.1
fn parse_args(args: &[String]) -> Result<ExtractMailsRequest, String> {
    let mut args = args.iter();

    let extract_type = match args.next().map(String::as_str) {
        Some("Bulk") => ExtractType::Bulk,
        Some("Increment") => ExtractType::Increment,
        Some("Sync") => ExtractType::Sync,
        _ => return Err("The first argument must be Bulk, Increment or Sync".to_string()),
    };

    let query = args
        .next()
        .filter(|query| !query.starts_with("--"))
        .ok_or("The second argument must be the Gmail query")?
        .clone();

    let mut count = None;
    let mut after = None;
    let mut before = None;

    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| format!("{flag} needs a value"))?;

        match flag.as_str() {
            "--count" => {
                count = Some(
                    value
                        .parse::<u32>()
                        .map_err(|_| "--count must be a whole number".to_string())?,
                );
            }
            "--after" => after = Some(value.clone()),
            "--before" => before = Some(value.clone()),
            other => return Err(format!("Unknown option {other}")),
        }
    }

    Ok(ExtractMailsRequest {
        extract_type,
        query,
        input_mails_count: count,
        input_date_after: after,
        input_date_before: before,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Builds the argument list from text pieces
    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| part.to_string()).collect()
    }

    // The mode and the query alone give a request without optional fields
    #[test]
    fn parses_mode_and_query() {
        let Ok(request) = parse_args(&args(&["Sync", "label:jobs"])) else {
            panic!("arguments should parse");
        };

        assert_eq!(request.extract_type, ExtractType::Sync);
        assert_eq!(request.query, "label:jobs");
        assert_eq!(request.input_mails_count, None);
        assert_eq!(request.input_date_after, None);
        assert_eq!(request.input_date_before, None);
    }

    // The count and both dates are read from their options
    #[test]
    fn parses_count_and_dates() {
        let Ok(request) = parse_args(&args(&[
            "Increment",
            "label:jobs",
            "--count",
            "5",
            "--after",
            "2026/11/1",
            "--before",
            "2026/11/9",
        ])) else {
            panic!("arguments should parse");
        };

        assert_eq!(request.extract_type, ExtractType::Increment);
        assert_eq!(request.input_mails_count, Some(5));
        assert_eq!(request.input_date_after.as_deref(), Some("2026/11/1"));
        assert_eq!(request.input_date_before.as_deref(), Some("2026/11/9"));
    }

    // An unknown mode is rejected
    #[test]
    fn rejects_an_unknown_mode() {
        assert!(parse_args(&args(&["Full", "label:jobs"])).is_err());
    }

    // A missing query is rejected, an option is not a query
    #[test]
    fn rejects_a_missing_query() {
        assert!(parse_args(&args(&["Bulk"])).is_err());
        assert!(parse_args(&args(&["Bulk", "--count", "5"])).is_err());
    }

    // A count that is not a number is rejected
    #[test]
    fn rejects_a_bad_count() {
        assert!(parse_args(&args(&["Bulk", "label:jobs", "--count", "five"])).is_err());
    }

    // An option without a value and an unknown option are rejected
    #[test]
    fn rejects_a_bad_option() {
        assert!(parse_args(&args(&["Bulk", "label:jobs", "--count"])).is_err());
        assert!(parse_args(&args(&["Bulk", "label:jobs", "--limit", "5"])).is_err());
    }
}
