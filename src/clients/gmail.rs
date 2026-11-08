use std::{
    sync::LazyLock,
    time::{Duration, Instant},
};

use reqwest::{Client, Response, StatusCode};
use serde::Deserialize;
use tokio::sync::{Mutex, OnceCell};

use crate::{
    config::env::env,
    db::google_oauth::{GoogleCreds, get_active_creds},
    errors::{
        api_error::ApiError,
        catalog::{forbidden, too_many_requests, unauthorized},
        code::ErrorCode,
    },
};

// Google token endpoint, checked against Google's OAuth documentation (D47)
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

// Base of the Gmail REST calls for the account the refresh token belongs to (D47)
const API_BASE: &str = "https://gmail.googleapis.com/gmail/v1/users/me";

// A cached access token counts as expired this long before Google says so
const EXPIRY_MARGIN: Duration = Duration::from_secs(60);

// Longest a single call to Google may take
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

// Shared http client, reuses connections between calls
static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .expect("Failed to build the Gmail http client")
});

// Session of the process, built from the database credentials on first use
static SESSION: OnceCell<GmailSession> = OnceCell::const_new();

// Access token with the moment it stops being usable, no Debug because it is a secret
struct CachedToken {
    token: String,
    expires_at: Instant,
}

// Reply of the Google token endpoint, no Debug because it holds a secret
#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

// Maps a Gmail or token endpoint status to the codes of SPEC section 5.5
fn map_status(status: StatusCode) -> ApiError {
    match status.as_u16() {
        400 | 401 => ApiError::from_entry(unauthorized::GMAIL_SESSION_EXPIRED),
        403 => ApiError::from_entry(forbidden::GMAIL_ACCESS_DENIED),
        429 => ApiError::from_entry(too_many_requests::GMAIL_RATE_LIMIT),
        other => ApiError::new(
            ErrorCode::InternalServerError,
            format!("Gmail API Error ({other})"),
        ),
    }
}

// Logs a failure without a status and returns the generic 500
fn call_failed(detail: impl std::fmt::Display) -> ApiError {
    tracing::error!(target: "Gmail", "{detail}");

    ApiError::new(ErrorCode::InternalServerError, "Gmail API request failed")
}

// Gmail connection for one set of credentials, caches the access token until it expires
pub struct GmailSession {
    token_url: String,
    api_base: String,
    creds: GoogleCreds,
    cache: Mutex<Option<CachedToken>>,
}

impl GmailSession {
    // Builds a session for the given endpoints and credentials, nothing is called yet
    pub fn new(token_url: &str, api_base: &str, creds: GoogleCreds) -> Self {
        Self {
            token_url: token_url.to_string(),
            api_base: api_base.to_string(),
            creds,
            cache: Mutex::new(None),
        }
    }

    // Exchanges the refresh token for a new access token
    async fn request_token(&self) -> Result<CachedToken, ApiError> {
        let response = CLIENT
            .post(&self.token_url)
            .form(&[
                ("client_id", self.creds.client_id.as_str()),
                ("client_secret", self.creds.client_secret.as_str()),
                ("refresh_token", self.creds.refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|error| call_failed(format!("token request failed: {error}")))?;

        // A rejected refresh token or client answers 400 or 401, both mean the session expired
        if !response.status().is_success() {
            tracing::error!(target: "Gmail", "token endpoint returned {}", response.status());

            return Err(map_status(response.status()));
        }

        let parsed: TokenResponse = response
            .json()
            .await
            .map_err(|error| call_failed(format!("token response is not valid: {error}")))?;

        Ok(CachedToken {
            token: parsed.access_token,
            expires_at: Instant::now() + Duration::from_secs(parsed.expires_in),
        })
    }

    // Returns the cached access token while it is still usable, otherwise (or when forced) a new one
    async fn access_token(&self, force_refresh: bool) -> Result<String, ApiError> {
        // The lock is held during the refresh, so concurrent calls share one refresh
        let mut cache = self.cache.lock().await;

        let usable = match cache.as_ref() {
            Some(cached)
                if !force_refresh && cached.expires_at > Instant::now() + EXPIRY_MARGIN =>
            {
                Some(cached.token.clone())
            }
            _ => None,
        };

        if let Some(token) = usable {
            return Ok(token);
        }

        let fresh = self.request_token().await?;
        let token = fresh.token.clone();

        *cache = Some(fresh);

        Ok(token)
    }

    // Sends one authorised GET to the Gmail API
    async fn send(
        &self,
        path: &str,
        query: &[(&str, &str)],
        token: &str,
    ) -> Result<Response, ApiError> {
        let url = format!(
            "{}/{}",
            self.api_base.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        CLIENT
            .get(url)
            .bearer_auth(token)
            .query(query)
            .send()
            .await
            .map_err(|error| call_failed(format!("Gmail request failed: {error}")))
    }

    // GETs a Gmail path, refreshes the token once on a 401 and maps every other failure (SPEC section 5.5)
    pub async fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<Response, ApiError> {
        let token = self.access_token(false).await?;
        let mut response = self.send(path, query, &token).await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            let token = self.access_token(true).await?;

            response = self.send(path, query, &token).await?;
        }

        if !response.status().is_success() {
            tracing::error!(target: "Gmail", "Gmail returned {}", response.status());

            return Err(map_status(response.status()));
        }

        Ok(response)
    }
}

// Returns the session of the process, reading the active credentials from the database on first use
pub async fn gmail_session() -> Result<&'static GmailSession, ApiError> {
    SESSION
        .get_or_try_init(|| async {
            let creds = get_active_creds(env().google_appid).await?;

            Ok(GmailSession::new(TOKEN_URL, API_BASE, creds))
        })
        .await
}
