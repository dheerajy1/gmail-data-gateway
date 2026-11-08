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
}
