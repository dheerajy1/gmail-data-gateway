use serde::Deserialize;
use validator::Validate;

// Default Azure SQL procedure that writes one raw applied job (SPEC section 10)
fn default_write_model_procedure() -> String {
    "[02sjobsData].[02spinsertRawAppliedJobs]".to_string()
}

// Default PostgreSQL procedure that applies committed records (SPEC section 10)
fn default_read_model_procedure() -> String {
    "sp_kafka_apply_committed_records".to_string()
}

// Runtime environment the app runs in, only these three values are accepted (D30)
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AppEnv {
    Development,
    Preview,
    Production,
}

impl AppEnv {
    // Returns the value as written in APP_ENV
    pub fn as_str(self) -> &'static str {
        match self {
            AppEnv::Development => "development",
            AppEnv::Preview => "preview",
            AppEnv::Production => "production",
        }
    }
}

// All environment variables, no Debug derive because most fields are secrets
#[derive(Clone, Deserialize, Validate)]
pub struct EnvSchema {
    // Application name
    #[validate(length(
        min = 1,
        max = 50,
        message = "APP_NAME must be between 1 and 50 characters"
    ))]
    pub app_name: String,

    // Runtime environment, development or production
    pub app_env: AppEnv,

    // Port the server listens on
    #[validate(range(min = 1, message = "APP_PORT must be a valid port number"))]
    pub app_port: u16,

    // Client id that callers send in the x-client-id header
    #[validate(length(
        min = 1,
        max = 50,
        message = "CLIENT_ID must be between 1 and 50 characters"
    ))]
    pub client_id: String,

    // Client secret that callers send in the x-client-secret header
    #[validate(length(
        min = 1,
        max = 50,
        message = "CLIENT_SECRET must be between 1 and 50 characters"
    ))]
    pub client_secret: String,

    // Secret used to sign and verify API login tokens
    #[validate(length(
        min = 1,
        max = 50,
        message = "JWT_SECRET must be between 1 and 50 characters"
    ))]
    pub jwt_secret: String,

    // Azure SQL server host name
    #[validate(length(
        min = 1,
        max = 50,
        message = "AZURE_DB_SERVER must be between 1 and 50 characters"
    ))]
    pub azure_db_server: String,

    // Azure SQL database name
    #[validate(length(
        min = 1,
        max = 50,
        message = "AZURE_DB_DATABASE must be between 1 and 50 characters"
    ))]
    pub azure_db_database: String,

    // Azure SQL user for portal login and public reads
    #[validate(length(
        min = 1,
        max = 50,
        message = "AZURE_DB_USER_PUBLIC must be between 1 and 50 characters"
    ))]
    pub azure_db_user_public: String,

    // Password of the public Azure SQL user
    #[validate(length(
        min = 1,
        max = 50,
        message = "AZURE_DB_PASSWORD_PUBLIC must be between 1 and 50 characters"
    ))]
    pub azure_db_password_public: String,

    // Azure SQL user for service writes such as raw job inserts
    #[validate(length(
        min = 1,
        max = 50,
        message = "AZURE_DB_USER_SERVICE must be between 1 and 50 characters"
    ))]
    pub azure_db_user_service: String,

    // Password of the service Azure SQL user
    #[validate(length(
        min = 1,
        max = 50,
        message = "AZURE_DB_PASSWORD_SERVICE must be between 1 and 50 characters"
    ))]
    pub azure_db_password_service: String,

    // Base url of the on-prem Kafka API gateway
    #[validate(url(message = "ONPREM_SERVER_URL must be a valid url"))]
    pub onprem_server_url: String,

    // Kafka api key sent to the gateway
    #[validate(length(
        min = 1,
        max = 50,
        message = "KAFKA_API_KEY must be between 1 and 50 characters"
    ))]
    pub kafka_api_key: String,

    // Kafka api secret sent to the gateway
    #[validate(length(
        min = 1,
        max = 300,
        message = "KAFKA_API_KEY_SECRET must be between 1 and 300 characters"
    ))]
    pub kafka_api_key_secret: String,

    // Kafka source client identifier
    #[validate(length(
        min = 1,
        max = 100,
        message = "SOURCE_CLIENT_ID must be between 1 and 100 characters"
    ))]
    pub source_client_id: String,

    // Kafka topic for jobs write-model ingest
    #[validate(length(
        min = 1,
        max = 100,
        message = "JOBS_WRITE_TOPIC must be between 1 and 100 characters"
    ))]
    pub jobs_write_topic: String,

    // Kafka dead letter topic for write-model failures
    #[validate(length(
        min = 1,
        max = 100,
        message = "JOBS_WRITE_DLQ_TOPIC must be between 1 and 100 characters"
    ))]
    pub jobs_write_dlq_topic: String,

    // Kafka topic for jobs read-model ingest
    #[validate(length(
        min = 1,
        max = 100,
        message = "JOBS_READ_TOPIC must be between 1 and 100 characters"
    ))]
    pub jobs_read_topic: String,

    // Kafka dead letter topic for read-model failures
    #[validate(length(
        min = 1,
        max = 100,
        message = "JOBS_READ_DLQ_TOPIC must be between 1 and 100 characters"
    ))]
    pub jobs_read_dlq_topic: String,

    // Kafka topic for jobs pipeline status events
    #[validate(length(
        min = 1,
        max = 100,
        message = "JOBS_STATUS_TOPIC must be between 1 and 100 characters"
    ))]
    pub jobs_status_topic: String,

    // Azure SQL write-model stored procedure name
    #[serde(default = "default_write_model_procedure")]
    #[validate(length(
        min = 1,
        max = 100,
        message = "JOBS_WRITE_MODEL_PROCEDURE must be between 1 and 100 characters"
    ))]
    pub jobs_write_model_procedure: String,

    // PostgreSQL read-model stored procedure name
    #[serde(default = "default_read_model_procedure")]
    #[validate(length(
        min = 1,
        max = 100,
        message = "JOBS_READ_MODEL_PROCEDURE must be between 1 and 100 characters"
    ))]
    pub jobs_read_model_procedure: String,

    // On-prem PostgreSQL host name
    #[validate(length(
        min = 1,
        max = 50,
        message = "ONPREM_DB_SERVER must be between 1 and 50 characters"
    ))]
    pub onprem_db_server: String,

    // On-prem PostgreSQL port
    #[validate(range(min = 1, message = "ONPREM_DB_PORT must be a valid port number"))]
    pub onprem_db_port: u16,

    // On-prem PostgreSQL database name
    #[validate(length(
        min = 1,
        max = 50,
        message = "ONPREM_DB_DATABASE must be between 1 and 50 characters"
    ))]
    pub onprem_db_database: String,

    // On-prem PostgreSQL service user
    #[validate(length(
        min = 1,
        max = 50,
        message = "ONPREM_DB_USER_SERVICE must be between 1 and 50 characters"
    ))]
    pub onprem_db_user_service: String,

    // Password of the on-prem PostgreSQL service user
    #[validate(length(
        min = 1,
        max = 50,
        message = "ONPREM_DB_PASSWORD_SERVICE must be between 1 and 50 characters"
    ))]
    pub onprem_db_password_service: String,

    // Id of the Google app whose refresh token is stored in Azure, read from Google_APPID
    #[validate(range(min = 1, max = 300, message = "Google_APPID must be between 1 and 300"))]
    pub google_appid: u16,
}
