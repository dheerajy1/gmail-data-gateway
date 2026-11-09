use jobs_api_gateway::db::azure::service_pool;
use serde::Deserialize;
use uuid::Uuid;

use crate::event::{AzureWriteRange, JobRecord};

// Notes the write-model stored procedure returns as json (SPEC section 9.1)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteModelNotes {
    // Number of mails inserted, zero is still a success
    pub rows_inserted: i64,

    // True when the same correlation id was already written
    #[serde(default)]
    pub replayed: bool,

    // First Azure id inserted, absent when no row was inserted or on a replay
    #[serde(default)]
    pub from_id: Option<i32>,

    // Last Azure id inserted, absent when no row was inserted or on a replay
    #[serde(default)]
    pub to_id: Option<i32>,
}

impl WriteModelNotes {
    // The Azure id range of the insert, only when both ends are there
    pub fn azure_write(&self) -> Option<AzureWriteRange> {
        Some(AzureWriteRange {
            from_id: self.from_id?,
            to_id: self.to_id?,
        })
    }
}

// The procedure name goes into the SQL text, so only safe characters are allowed (SPEC section 9.2)
fn assert_procedure_name(name: &str) -> Result<&str, String> {
    let valid = !name.is_empty()
        && name.len() <= 200
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '"' | '[' | ']'));

    if valid {
        Ok(name)
    } else {
        Err("Invalid write-model procedure name".to_string())
    }
}

// Runs the write-model stored procedure on the service pool, the name comes from the event
pub async fn execute_write_model(
    procedure_name: &str,
    correlation_id: &str,
    records: &[JobRecord],
) -> Result<WriteModelNotes, String> {
    let procedure = assert_procedure_name(procedure_name)?;

    let correlation_id = Uuid::parse_str(correlation_id).map_err(|error| error.to_string())?;
    let input = serde_json::to_string(records).map_err(|error| error.to_string())?;

    let mut connection = service_pool()
        .get()
        .await
        .map_err(|error| error.to_string())?;

    let row = connection
        .query(
            format!("EXEC {procedure} @correlationId = @P1, @input = @P2"),
            &[&correlation_id, &input],
        )
        .await
        .map_err(|error| error.to_string())?
        .into_row()
        .await
        .map_err(|error| error.to_string())?;

    let no_success = || "Write-model SP did not return SUCCESS with notes".to_string();

    let row = row.ok_or_else(no_success)?;

    let result = row
        .try_get::<&str, _>("result")
        .map_err(|error| error.to_string())?;
    let notes = row
        .try_get::<&str, _>("notes")
        .map_err(|error| error.to_string())?;

    // The procedure must answer SUCCESS with a notes json
    let (Some("SUCCESS"), Some(notes)) = (result, notes) else {
        return Err(no_success());
    };

    serde_json::from_str::<WriteModelNotes>(notes)
        .map_err(|_| "Write-model SP notes missing rowsInserted".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Plain and schema qualified procedure names are accepted
    #[test]
    fn accepts_safe_procedure_names() {
        assert_eq!(assert_procedure_name("write_proc"), Ok("write_proc"));
        assert_eq!(
            assert_procedure_name("[02sjobsData].[02spinsertRawAppliedJobs]"),
            Ok("[02sjobsData].[02spinsertRawAppliedJobs]")
        );
        assert_eq!(assert_procedure_name("dbo.usp_write"), Ok("dbo.usp_write"));
    }

    // An empty name is rejected
    #[test]
    fn rejects_empty_procedure_name() {
        assert!(assert_procedure_name("").is_err());
    }

    // Names with characters that could change the SQL text are rejected
    #[test]
    fn rejects_unsafe_procedure_names() {
        assert!(assert_procedure_name("proc; DROP TABLE jobs").is_err());
        assert!(assert_procedure_name("proc--comment").is_err());
        assert!(assert_procedure_name("proc'name").is_err());
        assert!(assert_procedure_name("proc name").is_err());
    }

    // A name over 200 characters is rejected
    #[test]
    fn rejects_too_long_procedure_name() {
        assert!(assert_procedure_name(&"a".repeat(201)).is_err());
        assert!(assert_procedure_name(&"a".repeat(200)).is_ok());
    }

    // Notes from the stored procedure are read with the camelCase names and extra keys are ignored
    #[test]
    fn parses_write_model_notes() {
        let Ok(notes) = serde_json::from_str::<WriteModelNotes>(
            r#"{"rowsInserted": 3, "message": "ok", "replayed": true}"#,
        ) else {
            panic!("notes should parse");
        };

        assert_eq!(notes.rows_inserted, 3);
        assert!(notes.replayed);
    }

    // The replayed flag defaults to false when the procedure leaves it out
    #[test]
    fn replayed_defaults_to_false() {
        let Ok(notes) = serde_json::from_str::<WriteModelNotes>(r#"{"rowsInserted": 0}"#) else {
            panic!("notes should parse");
        };

        assert_eq!(notes.rows_inserted, 0);
        assert!(!notes.replayed);
    }

    // Notes without rowsInserted are rejected
    #[test]
    fn rejects_notes_without_rows_inserted() {
        assert!(serde_json::from_str::<WriteModelNotes>(r#"{"replayed": false}"#).is_err());
    }

    // The id range is read from the notes when the procedure inserted rows
    #[test]
    fn notes_give_the_azure_range() {
        let Ok(notes) = serde_json::from_str::<WriteModelNotes>(
            r#"{"rowsInserted": 3, "fromId": 295, "toId": 297, "replayed": false}"#,
        ) else {
            panic!("notes should parse");
        };

        assert_eq!(
            notes.azure_write(),
            Some(AzureWriteRange {
                from_id: 295,
                to_id: 297
            })
        );
    }

    // No range when no row was inserted, when the notes come from a replay or when one end is missing
    #[test]
    fn notes_without_ids_give_no_range() {
        let Ok(empty) = serde_json::from_str::<WriteModelNotes>(r#"{"rowsInserted": 0}"#) else {
            panic!("notes should parse");
        };
        let Ok(half) =
            serde_json::from_str::<WriteModelNotes>(r#"{"rowsInserted": 1, "fromId": 4}"#)
        else {
            panic!("notes should parse");
        };

        assert_eq!(empty.azure_write(), None);
        assert_eq!(half.azure_write(), None);
    }
}
