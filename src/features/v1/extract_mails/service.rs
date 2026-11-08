use super::{
    model::{ExtractMailsData, ExtractMailsRequest, ExtractType},
    publisher::{PublishConfig, publish_chunks},
    query::{BuiltQuery, build_query},
    reader::{MailRecord, ReadPacing, read_mails},
};
use crate::{
    clients::{
        gmail::gmail_session,
        kafka_gateway::{GatewayTarget, gateway_target_from_env},
    },
    config::env::env,
    db::applied_jobs::get_last_applied_unix,
    errors::api_error::ApiError,
};

// Status when at least one chunk was published
const STATUS_ACCEPTED: &str = "ACCEPTED";

// Status when the query found no mails, nothing is published (SPEC section 5.6)
const STATUS_NO_NEW_RECORDS: &str = "NO_NEW_RECORDS";

// Message and data of a successful extract run
#[derive(Debug)]
pub struct ExtractOutcome {
    // Text of the success envelope
    pub message: String,

    // Data of the success envelope
    pub data: ExtractMailsData,
}

// Validates a request and builds its Gmail query, Sync reads the last stored date first
pub async fn prepare_query(request: &ExtractMailsRequest) -> Result<BuiltQuery, ApiError> {
    request.validate()?;

    let last_stored_unix = match request.extract_type {
        ExtractType::Sync => Some(get_last_applied_unix().await?),
        ExtractType::Bulk | ExtractType::Increment => None,
    };

    build_query(request, last_stored_unix)
}

// Publishes the mails read by a run and builds the response, no mails means nothing is published (SPEC section 5.6)
pub async fn publish_extract(
    extract_type: ExtractType,
    mails: Vec<MailRecord>,
    config: &PublishConfig<'_>,
    gateway: &GatewayTarget<'_>,
) -> Result<ExtractOutcome, ApiError> {
    if mails.is_empty() {
        return Ok(ExtractOutcome {
            message: "No messages found for the given query.".to_string(),
            data: ExtractMailsData {
                correlation_ids: Vec::new(),
                record_count: 0,
                chunk_count: 0,
                status: STATUS_NO_NEW_RECORDS.to_string(),
            },
        });
    }

    let record_count = mails.len();
    let correlation_ids = publish_chunks(mails, config, gateway).await?;
    let chunk_count = correlation_ids.len();

    Ok(ExtractOutcome {
        message: format!(
            "{} accepted. Write-model events published: {chunk_count}. Azure and PostgreSQL processing is asynchronous.",
            extract_type.as_str()
        ),
        data: ExtractMailsData {
            correlation_ids,
            record_count,
            chunk_count,
            status: STATUS_ACCEPTED.to_string(),
        },
    })
}

// Runs one extract: validates the request, reads the mails from Gmail and publishes them to the gateway (SPEC section 5)
pub async fn extract_mails(request: &ExtractMailsRequest) -> Result<ExtractOutcome, ApiError> {
    let built = prepare_query(request).await?;
    let session = gmail_session().await?;
    let mails = read_mails(session, &built.q, built.limit, ReadPacing::STANDARD).await?;

    let env = env();

    let config = PublishConfig {
        topic: &env.jobs_write_topic,
        write_model: &env.jobs_write_model_procedure,
        read_model: &env.jobs_read_model_procedure,
        source_client_id: &env.source_client_id,
    };

    publish_extract(
        request.extract_type,
        mails,
        &config,
        &gateway_target_from_env(),
    )
    .await
}
