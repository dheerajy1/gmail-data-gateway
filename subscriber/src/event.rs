use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

// The write and read model procedures an event is processed by
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobModels {
    // Stored procedure of the write model
    pub write_model: String,

    // Stored procedure of the read model
    pub read_model: String,
}

// One mail inside an event
#[derive(Debug, Clone, Deserialize)]
pub struct JobRecord {
    // Gmail message id
    pub id: String,

    // Mail date in India time
    pub date: String,

    // Sender
    pub from: String,

    // Subject
    pub subject: String,

    // Gmail snippet
    pub snippet: String,
}

// The jobs event published by the API and consumed by both stages (SPEC section 6)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobsEvent {
    // UUIDv7 that follows the chunk through the whole pipeline
    pub correlation_id: String,

    // Procedures that process the records
    pub job: JobModels,

    // Id of the client that published the event
    pub source_client_id: String,

    // The mails of the chunk, at least one
    pub records: Vec<JobRecord>,
}

// True when the text is a UUID of version 7
pub fn is_uuid_v7(text: &str) -> bool {
    Uuid::parse_str(text).is_ok_and(|uuid| uuid.get_version_num() == 7)
}

// Describes a json shape error without echoing values, so mail content never reaches a log (rule 4)
// Missing fields and unknown variants name only the schema, everything else gets a generic text
pub fn describe_shape(error: &serde_json::Error) -> String {
    let text = error.to_string();

    if text.starts_with("missing field") || text.starts_with("unknown variant") {
        text
    } else {
        "a field has the wrong type or shape".to_string()
    }
}

impl JobsEvent {
    // Validates a payload and returns the event, the error never contains mail content
    pub fn parse(payload: &Value) -> Result<Self, String> {
        let event: Self = serde_json::from_value(payload.clone())
            .map_err(|error| format!("invalid jobs event: {}", describe_shape(&error)))?;

        if !is_uuid_v7(&event.correlation_id) {
            return Err("invalid jobs event: correlationId must be a uuid v7".to_string());
        }

        if event.job.write_model.is_empty() || event.job.read_model.is_empty() {
            return Err("invalid jobs event: job models must not be empty".to_string());
        }

        if event.source_client_id.is_empty() {
            return Err("invalid jobs event: sourceClientId must not be empty".to_string());
        }

        if event.records.is_empty() {
            return Err("invalid jobs event: records must not be empty".to_string());
        }

        let incomplete = event.records.iter().any(|record| {
            record.id.is_empty()
                || record.date.is_empty()
                || record.from.is_empty()
                || record.subject.is_empty()
                || record.snippet.is_empty()
        });

        if incomplete {
            return Err("invalid jobs event: every record field must be filled".to_string());
        }

        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    // A UUIDv7 used by the tests
    const CORRELATION_ID: &str = "0190a3c0-7b1e-7000-8000-000000000001";

    // A valid event payload
    fn valid() -> Value {
        json!({
            "correlationId": CORRELATION_ID,
            "job": { "writeModel": "write-proc", "readModel": "read-proc" },
            "sourceClientId": "source-1",
            "records": [{
                "id": "m1",
                "date": "2026-11-02T15:45:30+05:30",
                "from": "Sender <sender@example.com>",
                "subject": "Subject",
                "snippet": "Snippet",
            }],
        })
    }

    // Parses the payload and returns the error text, panics when it is valid
    fn error_of(payload: &Value) -> String {
        match JobsEvent::parse(payload) {
            Ok(_) => panic!("payload should be invalid"),
            Err(error) => error,
        }
    }

    // A valid payload parses with all its fields
    #[test]
    fn parses_a_valid_event() {
        let Ok(event) = JobsEvent::parse(&valid()) else {
            panic!("valid event rejected");
        };

        assert_eq!(event.correlation_id, CORRELATION_ID);
        assert_eq!(event.job.write_model, "write-proc");
        assert_eq!(event.job.read_model, "read-proc");
        assert_eq!(event.source_client_id, "source-1");
        assert_eq!(event.records.len(), 1);
        assert_eq!(event.records[0].id, "m1");
    }

    // Only UUIDv7 correlation ids are accepted
    #[test]
    fn rejects_a_correlation_id_that_is_not_uuid_v7() {
        let mut payload = valid();
        payload["correlationId"] = json!("550e8400-e29b-41d4-a716-446655440000");

        assert!(error_of(&payload).contains("correlationId"));

        payload["correlationId"] = json!("not-a-uuid");

        assert!(error_of(&payload).contains("correlationId"));
    }

    // Empty model names, source client and records are rejected
    #[test]
    fn rejects_empty_values() {
        let mut payload = valid();
        payload["job"]["writeModel"] = json!("");

        assert!(error_of(&payload).contains("job models"));

        let mut payload = valid();
        payload["job"]["readModel"] = json!("");

        assert!(error_of(&payload).contains("job models"));

        let mut payload = valid();
        payload["sourceClientId"] = json!("");

        assert!(error_of(&payload).contains("sourceClientId"));

        let mut payload = valid();
        payload["records"] = json!([]);

        assert!(error_of(&payload).contains("records"));
    }

    // Every record field must be filled
    #[test]
    fn rejects_a_record_with_an_empty_field() {
        for field in ["id", "date", "from", "subject", "snippet"] {
            let mut payload = valid();
            payload["records"][0][field] = json!("");

            assert!(error_of(&payload).contains("record field"), "field {field}");
        }
    }

    // A missing field is named in the error
    #[test]
    fn names_a_missing_field() {
        let mut payload = valid();

        if let Some(object) = payload.as_object_mut() {
            object.remove("records");
        }

        assert!(error_of(&payload).contains("missing field `records`"));
    }

    // A wrong type gives a generic text and never echoes the value
    #[test]
    fn wrong_type_does_not_echo_the_value() {
        let mut payload = valid();
        payload["records"] = json!("secret-mail-text");

        let error = error_of(&payload);

        assert!(!error.contains("secret-mail-text"));
        assert!(error.contains("wrong type"));
    }

    // The UUIDv7 check accepts v7 and rejects other versions
    #[test]
    fn uuid_v7_check() {
        assert!(is_uuid_v7(CORRELATION_ID));
        assert!(!is_uuid_v7("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!is_uuid_v7(""));
    }
}
