use jobs_api_gateway::config::env::Env;

// A complete valid set of variables
fn valid_pairs() -> Vec<(String, String)> {
    [
        ("APP_NAME", "jobs-api-gateway"),
        ("APP_ENV", "production"),
        ("APP_PORT", "8091"),
        ("CLIENT_ID", "client"),
        ("CLIENT_SECRET", "secret"),
        ("JWT_SECRET", "jwt-secret"),
        ("AZURE_DB_SERVER", "server"),
        ("AZURE_DB_DATABASE", "database"),
        ("AZURE_DB_USER_PUBLIC", "public-user"),
        ("AZURE_DB_PASSWORD_PUBLIC", "public-password"),
        ("AZURE_DB_USER_SERVICE", "service-user"),
        ("AZURE_DB_PASSWORD_SERVICE", "service-password"),
        ("ONPREM_SERVER_URL", "http://127.0.0.1:9"),
        ("KAFKA_API_KEY", "kafka-key"),
        ("KAFKA_API_KEY_SECRET", "kafka-secret"),
        ("SOURCE_CLIENT_ID", "source"),
        ("JOBS_WRITE_TOPIC", "write"),
        ("JOBS_WRITE_DLQ_TOPIC", "write-dlq"),
        ("JOBS_READ_TOPIC", "read"),
        ("JOBS_READ_DLQ_TOPIC", "read-dlq"),
        ("JOBS_STATUS_TOPIC", "status"),
        ("ONPREM_DB_SERVER", "onprem"),
        ("ONPREM_DB_PORT", "5432"),
        ("ONPREM_DB_DATABASE", "onprem-db"),
        ("ONPREM_DB_USER_SERVICE", "onprem-user"),
        ("ONPREM_DB_PASSWORD_SERVICE", "onprem-password"),
        ("Google_APPID", "7"),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_string(), value.to_string()))
    .collect()
}

// Returns the valid set with one variable replaced
fn with(name: &str, value: &str) -> Vec<(String, String)> {
    let mut pairs = valid_pairs();

    for pair in pairs.iter_mut() {
        if pair.0 == name {
            pair.1 = value.to_string();
        }
    }

    pairs
}

// Returns the valid set without one variable
fn without(name: &str) -> Vec<(String, String)> {
    valid_pairs()
        .into_iter()
        .filter(|pair| pair.0 != name)
        .collect()
}

// Extracts the error text, the Env type has no Debug so Result::unwrap_err is not used
fn error_of(pairs: Vec<(String, String)>) -> String {
    match Env::from_pairs(pairs) {
        Ok(_) => String::new(),
        Err(error) => error,
    }
}

// A complete valid set loads, and the two procedure variables fall back to their defaults
#[test]
fn valid_env_loads_with_defaults() {
    let Ok(loaded) = Env::from_pairs(valid_pairs()) else {
        panic!("a valid env must load");
    };

    assert_eq!(loaded.app_env.as_str(), "production");
    assert_eq!(
        loaded.jobs_write_model_procedure,
        "[02sjobsData].[02spinsertRawAppliedJobs]"
    );
    assert_eq!(
        loaded.jobs_read_model_procedure,
        "sp_kafka_apply_committed_records"
    );
    assert_eq!(loaded.google_appid, 7);
}

// A missing variable stops loading and names the variable
#[test]
fn missing_variable_names_it() {
    assert!(error_of(without("APP_PORT")).contains("APP_PORT"));
    assert!(error_of(without("JWT_SECRET")).contains("JWT_SECRET"));
    assert!(error_of(without("Google_APPID")).contains("Google_APPID"));
}

// APP_ENV accepts only development, preview and production
#[test]
fn app_env_rejects_other_values() {
    assert!(error_of(with("APP_ENV", "staging")).contains("APP_ENV"));
    assert!(Env::from_pairs(with("APP_ENV", "development")).is_ok());
    assert!(Env::from_pairs(with("APP_ENV", "preview")).is_ok());
    assert!(Env::from_pairs(with("APP_ENV", "production")).is_ok());
}

// Out of range numbers stop loading and name the variable
#[test]
fn out_of_range_numbers_name_the_variable() {
    assert!(error_of(with("APP_PORT", "0")).contains("APP_PORT"));
    assert!(error_of(with("ONPREM_DB_PORT", "0")).contains("ONPREM_DB_PORT"));
    assert!(error_of(with("Google_APPID", "0")).contains("Google_APPID"));
    assert!(error_of(with("Google_APPID", "301")).contains("Google_APPID"));
}

// A value that is not a number stops loading and names the variable
#[test]
fn non_numeric_port_names_the_variable() {
    assert!(error_of(with("APP_PORT", "abc")).contains("APP_PORT"));
    assert!(error_of(with("Google_APPID", "abc")).contains("Google_APPID"));
}

// Too long values and bad urls stop loading and name the variable
#[test]
fn length_and_url_rules_name_the_variable() {
    assert!(error_of(with("APP_NAME", "")).contains("APP_NAME"));
    assert!(error_of(with("KAFKA_API_KEY", &"k".repeat(51))).contains("KAFKA_API_KEY"));
    assert!(error_of(with("ONPREM_SERVER_URL", "not a url")).contains("ONPREM_SERVER_URL"));
}

// The error text never contains the secret value that failed
#[test]
fn error_text_does_not_contain_secret_values() {
    let secret = "s".repeat(51);
    let error = error_of(with("CLIENT_SECRET", &secret));

    assert!(error.contains("CLIENT_SECRET"));
    assert!(!error.contains(&secret));
}
