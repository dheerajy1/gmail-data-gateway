use serde::Deserialize;
use tiberius::error::Error as SqlError;

use crate::{
    db::azure::service_pool,
    errors::{
        api_error::ApiError,
        catalog::{internal_server_error, not_found, unauthorized},
    },
};

// Error numbers raised by [05spgetActiveGoogleOAuthCreds]
const CLIENT_NOT_FOUND_ERROR: u32 = 50002;
const NO_ACTIVE_TOKEN_ERROR: u32 = 50003;

// Error number raised by [04spupsertGoogleOAuthToken] for an empty refresh token
const EMPTY_TOKEN_ERROR: u32 = 50001;

// Active Google OAuth credentials, no Debug because it holds secrets
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleCreds {
    // OAuth client id of the Google app
    pub client_id: String,

    // OAuth client secret of the Google app
    pub client_secret: String,

    // Long-lived refresh token that is exchanged for access tokens
    pub refresh_token: String,
}

// Maps a database error of the procedure to the listed codes, everything else is EXTRACT_FAILED
fn map_error(error: SqlError) -> ApiError {
    match &error {
        SqlError::Server(token) if token.code() == CLIENT_NOT_FOUND_ERROR => {
            ApiError::from_entry(not_found::CLIENT_ID_NOT_FOUND)
        }
        SqlError::Server(token) if token.code() == NO_ACTIVE_TOKEN_ERROR => {
            ApiError::from_entry(unauthorized::NO_ACTIVE_CREDENTIALS)
        }
        _ => {
            tracing::error!(target: "Db", "credentials query failed: {error}");

            ApiError::from_entry(internal_server_error::EXTRACT_FAILED)
        }
    }
}

// Reads the active credentials of a Google app from [05spgetActiveGoogleOAuthCreds]
pub async fn get_active_creds(app_id: u16) -> Result<GoogleCreds, ApiError> {
    let app_id = i32::from(app_id);

    let mut connection = service_pool().get().await.map_err(|error| {
        tracing::error!(target: "Db", "service pool failed: {error}");

        ApiError::from_entry(internal_server_error::EXTRACT_FAILED)
    })?;

    let row = connection
        .query(
            "EXEC [02sjobsData].[05spgetActiveGoogleOAuthCreds] @id = @P1",
            &[&app_id],
        )
        .await
        .map_err(map_error)?
        .into_row()
        .await
        .map_err(map_error)?;

    // The procedure returns the three fields as one json text in the notes column
    let creds = row
        .as_ref()
        .and_then(|row| row.get::<&str, _>("notes"))
        .and_then(|notes| serde_json::from_str::<GoogleCreds>(notes).ok());

    let Some(creds) = creds else {
        tracing::error!(target: "Db", "credentials result has an unexpected format");

        return Err(ApiError::from_entry(internal_server_error::EXTRACT_FAILED));
    };

    Ok(creds)
}

// Client credentials of a Google app, no Debug because it holds a secret
#[derive(Clone)]
pub struct GoogleApp {
    // OAuth client id of the Google app
    pub client_id: String,

    // OAuth client secret of the Google app
    pub client_secret: String,
}

// Notes of [04spupsertGoogleOAuthToken], only the app id is used
#[derive(Debug, Deserialize)]
struct UpsertNotes {
    // Id of the app the new token belongs to
    #[serde(rename = "appID")]
    app_id: i32,
}

// Turns a database error of the token tooling into a message without SQL text
fn tool_error(error: SqlError) -> String {
    match &error {
        SqlError::Server(token) if token.code() == EMPTY_TOKEN_ERROR => {
            "The refresh token is empty".to_string()
        }
        SqlError::Server(token) if token.code() == CLIENT_NOT_FOUND_ERROR => {
            "Google app not found for Google_APPID".to_string()
        }
        _ => format!("Database error: {error}"),
    }
}

// Reads the client id and secret of a Google app, the app needs no active token yet
pub async fn get_app(app_id: u16) -> Result<GoogleApp, String> {
    let app_id = i32::from(app_id);

    let mut connection = service_pool()
        .get()
        .await
        .map_err(|error| format!("Service pool failed: {error}"))?;

    let row = connection
        .query(
            "SELECT clientId, clientSecret FROM [02sjobsData].[05tgoogleOAuthApps] WHERE ID = @P1",
            &[&app_id],
        )
        .await
        .map_err(tool_error)?
        .into_row()
        .await
        .map_err(tool_error)?;

    let app = row.as_ref().and_then(|row| {
        Some(GoogleApp {
            client_id: row.get::<&str, _>("clientId")?.to_string(),
            client_secret: row.get::<&str, _>("clientSecret")?.to_string(),
        })
    });

    app.ok_or_else(|| "Google app not found for Google_APPID".to_string())
}

// Saves a new refresh token for a Google app through [04spupsertGoogleOAuthToken], returns the app id
pub async fn upsert_refresh_token(app_id: u16, refresh_token: &str) -> Result<i32, String> {
    let app_id = i32::from(app_id);
    let refresh_token = refresh_token.to_string();

    let mut connection = service_pool()
        .get()
        .await
        .map_err(|error| format!("Service pool failed: {error}"))?;

    let row = connection
        .query(
            "EXEC [02sjobsData].[04spupsertGoogleOAuthToken] @id = @P1, @refreshToken = @P2",
            &[&app_id, &refresh_token],
        )
        .await
        .map_err(tool_error)?
        .into_row()
        .await
        .map_err(tool_error)?;

    let no_success = || "Token procedure did not return SUCCESS with notes".to_string();

    let row = row.ok_or_else(no_success)?;

    let result = row
        .try_get::<&str, _>("result")
        .map_err(|error| error.to_string())?;
    let notes = row
        .try_get::<&str, _>("notes")
        .map_err(|error| error.to_string())?;

    let (Some("SUCCESS"), Some(notes)) = (result, notes) else {
        return Err(no_success());
    };

    serde_json::from_str::<UpsertNotes>(notes)
        .map(|notes| notes.app_id)
        .map_err(|_| no_success())
}

#[cfg(test)]
mod tests {
    use super::*;

    // The json text of the notes column parses into the three fields
    #[test]
    fn notes_json_parses() {
        let notes = r#"{"clientId":"id-1","clientSecret":"secret-1","refreshToken":"refresh-1"}"#;
        let creds: GoogleCreds = serde_json::from_str(notes).unwrap();

        assert_eq!(creds.client_id, "id-1");
        assert_eq!(creds.client_secret, "secret-1");
        assert_eq!(creds.refresh_token, "refresh-1");
    }

    // A notes value without the refresh token does not parse
    #[test]
    fn notes_without_refresh_token_do_not_parse() {
        let notes = r#"{"clientId":"id-1","clientSecret":"secret-1"}"#;

        assert!(serde_json::from_str::<GoogleCreds>(notes).is_err());
    }

    // The notes of the token procedure give the app id under appID
    #[test]
    fn upsert_notes_parse_app_id() {
        let Ok(notes) = serde_json::from_str::<UpsertNotes>(r#"{"appID":7}"#) else {
            panic!("notes should parse");
        };

        assert_eq!(notes.app_id, 7);
    }

    // Notes without appID do not parse
    #[test]
    fn upsert_notes_without_app_id_do_not_parse() {
        assert!(serde_json::from_str::<UpsertNotes>(r#"{"appId":7}"#).is_err());
    }
}
