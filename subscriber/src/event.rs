use serde::{Deserialize, Serialize};
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

// Azure ids given to the rows of one write-model event, Azure hands them out oldest mail first (D57)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureWriteRange {
    // First Azure id inserted
    pub from_id: i32,

    // Last Azure id inserted
    pub to_id: i32,
}

// One mail inside an event, it is serialized again as the input of the stored procedures
#[derive(Debug, Clone, Deserialize, Serialize)]
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

    // Azure id range, added by the write-model handler for the read-model topic
    #[serde(default)]
    pub azure_write: Option<AzureWriteRange>,
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

        if event
            .azure_write
            .is_some_and(|range| range.from_id < 1 || range.to_id < range.from_id)
        {
            return Err("invalid jobs event: azureWrite is not a valid id range".to_string());
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

// Returns a copy of the payload with the Azure id range set, or without it when there is none
// The payload keeps every other field exactly as it came
pub fn with_azure_write(payload: &Value, range: Option<AzureWriteRange>) -> Result<Value, String> {
    let mut next = payload.clone();

    let Some(object) = next.as_object_mut() else {
        return Err("invalid jobs event: payload is not an object".to_string());
    };

    match range {
        Some(range) => {
            let range = serde_json::to_value(range).map_err(|error| error.to_string())?;

            object.insert("azureWrite".to_string(), range);
        }
        None => {
            object.remove("azureWrite");
        }
    }

    Ok(next)
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

    // The Azure id range is absent on a write-model event
    #[test]
    fn azure_write_is_optional() {
        let Ok(event) = JobsEvent::parse(&valid()) else {
            panic!("event should parse");
        };

        assert_eq!(event.azure_write, None);
    }

    // The Azure id range is read with camelCase names
    #[test]
    fn azure_write_is_read() {
        let mut payload = valid();
        payload["azureWrite"] = json!({ "fromId": 295, "toId": 297 });

        let Ok(event) = JobsEvent::parse(&payload) else {
            panic!("event should parse");
        };

        assert_eq!(
            event.azure_write,
            Some(AzureWriteRange {
                from_id: 295,
                to_id: 297
            })
        );
    }

    // A reversed range or an id below 1 is rejected
    #[test]
    fn azure_write_rejects_a_bad_range() {
        let mut reversed = valid();
        reversed["azureWrite"] = json!({ "fromId": 9, "toId": 3 });

        let mut zero = valid();
        zero["azureWrite"] = json!({ "fromId": 0, "toId": 3 });

        assert!(JobsEvent::parse(&reversed).is_err());
        assert!(JobsEvent::parse(&zero).is_err());
    }

    // The range is added to a copy and the other fields stay
    #[test]
    fn with_azure_write_adds_the_range() {
        let payload = valid();

        let Ok(next) = with_azure_write(
            &payload,
            Some(AzureWriteRange {
                from_id: 1,
                to_id: 5,
            }),
        ) else {
            panic!("range should be added");
        };

        assert_eq!(next["azureWrite"], json!({ "fromId": 1, "toId": 5 }));
        assert_eq!(next["correlationId"], payload["correlationId"]);
        assert!(payload.get("azureWrite").is_none());
    }

    // Without a range an old azureWrite value is removed
    #[test]
    fn with_azure_write_removes_a_stale_range() {
        let mut payload = valid();
        payload["azureWrite"] = json!({ "fromId": 1, "toId": 5 });

        let Ok(next) = with_azure_write(&payload, None) else {
            panic!("range should be removed");
        };

        assert!(next.get("azureWrite").is_none());
    }

    // A payload that is not an object is rejected
    #[test]
    fn with_azure_write_rejects_a_non_object() {
        assert!(with_azure_write(&json!([1, 2]), None).is_err());
    }
}
