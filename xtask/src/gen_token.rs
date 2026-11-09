use std::io::{self, Write};
use std::process::exit;
use std::time::Duration;

use jobs_api_gateway::{
    db::google_oauth::{GoogleApp, get_app, upsert_refresh_token},
    logging::iso_now_ist,
};
use reqwest::{Client, Url};
use serde::Deserialize;

// Google page where the person gives consent
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";

// Google endpoint that exchanges the code for tokens
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

// Redirect of the old generate-token.ts, the page does not have to exist, only the code in its address matters
const REDIRECT_URI: &str = "http://localhost:3000";

// Scopes the stored token is allowed to use: Gmail read and Drive (D14)
const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/gmail.readonly",
    "https://www.googleapis.com/auth/drive",
];

// Longest the token request may take
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

// Answer of the token endpoint, only the fields used here
#[derive(Deserialize)]
struct TokenResponse {
    // Long-lived token, Google sends it only on a consent that asked for offline access
    refresh_token: Option<String>,

    // Short error name when Google rejects the request
    error: Option<String>,
}

// Runs the interactive consent flow for the Google app of Google_APPID and stores the new refresh token
pub fn run(app_id: u16) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("[genToken:Error]\t Failed to start the runtime: {error}");
            exit(1);
        }
    };

    if let Err(error) = runtime.block_on(flow(app_id)) {
        eprintln!("[genToken:Error]\t {error}");
        exit(1);
    }
}

// The whole flow: read the app, print the consent URL, read the code, exchange it, store the token
async fn flow(app_id: u16) -> Result<(), String> {
    let app = get_app(app_id).await?;

    let consent_url = consent_url(&app.client_id)?;

    println!("Authorize this app by visiting this URL:\n{consent_url}");

    print!("Enter the code from that page here: ");
    io::stdout().flush().map_err(|error| error.to_string())?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|error| error.to_string())?;

    let code = extract_code(&input)?;

    let refresh_token = exchange_code(&app, &code).await?;

    // Tokens are never printed in full (F40)
    println!("\nYour Refresh Token: {}", mask(&refresh_token));

    let stored_app_id = upsert_refresh_token(app_id, &refresh_token).await?;

    println!(
        "{}\t [genToken:Log]\t Successfully inserted new refresh token for google app id {stored_app_id}",
        iso_now_ist()
    );

    Ok(())
}

// Builds the consent URL, offline access and a forced consent so Google returns a refresh token
fn consent_url(client_id: &str) -> Result<String, String> {
    let scope = SCOPES.join(" ");

    Url::parse_with_params(
        AUTH_URL,
        [
            ("client_id", client_id),
            ("redirect_uri", REDIRECT_URI),
            ("response_type", "code"),
            ("access_type", "offline"),
            ("prompt", "consent"),
            ("scope", scope.as_str()),
        ],
    )
    .map(String::from)
    .map_err(|_| "Failed to build the consent URL".to_string())
}

// Takes the code from what the person pasted: the bare code, a percent-encoded code or the whole redirect address
fn extract_code(input: &str) -> Result<String, String> {
    let input = input.trim();

    if input.is_empty() {
        return Err("No code was entered".to_string());
    }

    // A pasted redirect address, take its code parameter
    if input.starts_with("http") {
        let url = Url::parse(input).map_err(|_| "The pasted address is not valid".to_string())?;

        return url
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.into_owned())
            .ok_or_else(|| "The pasted address has no code parameter".to_string());
    }

    // A code copied from the address bar is still percent-encoded
    if input.contains('%') {
        let decoded = Url::parse(&format!("http://localhost/?code={input}"))
            .ok()
            .and_then(|url| {
                url.query_pairs()
                    .find(|(key, _)| key == "code")
                    .map(|(_, value)| value.into_owned())
            });

        if let Some(decoded) = decoded {
            return Ok(decoded);
        }
    }

    Ok(input.to_string())
}

// Shows the start and the end of a token with its length, never the whole token
fn mask(token: &str) -> String {
    let length = token.chars().count();

    if length <= 12 {
        return format!("**** ({length} chars)");
    }

    let start: String = token.chars().take(4).collect();
    let end: String = token.chars().skip(length - 4).collect();

    format!("{start}...{end} ({length} chars)")
}

// Exchanges the consent code for tokens and returns the refresh token
async fn exchange_code(app: &GoogleApp, code: &str) -> Result<String, String> {
    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| format!("Failed to build the http client: {error}"))?;

    let response = client
        .post(TOKEN_URL)
        .form(&[
            ("code", code),
            ("client_id", app.client_id.as_str()),
            ("client_secret", app.client_secret.as_str()),
            ("redirect_uri", REDIRECT_URI),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|error| format!("Token request failed: {error}"))?;

    let status = response.status();

    let parsed: TokenResponse = response
        .json()
        .await
        .map_err(|_| format!("Token response is not valid (status {status})"))?;

    if !status.is_success() {
        return Err(format!(
            "Google rejected the code (status {status}): {}",
            parsed.error.as_deref().unwrap_or("unknown error")
        ));
    }

    parsed
        .refresh_token
        .filter(|token| !token.is_empty())
        .ok_or_else(|| "Google returned no refresh token".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // The consent URL carries the client id, redirect, offline access, forced consent and both scopes
    #[test]
    fn consent_url_has_all_parameters() {
        let Ok(url) = consent_url("client-1") else {
            panic!("url should build");
        };
        let Ok(url) = Url::parse(&url) else {
            panic!("url should parse");
        };

        let pairs: Vec<(String, String)> = url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect();

        let get = |name: &str| {
            pairs
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.as_str())
        };

        assert_eq!(url.host_str(), Some("accounts.google.com"));
        assert_eq!(get("client_id"), Some("client-1"));
        assert_eq!(get("redirect_uri"), Some("http://localhost:3000"));
        assert_eq!(get("response_type"), Some("code"));
        assert_eq!(get("access_type"), Some("offline"));
        assert_eq!(get("prompt"), Some("consent"));
        assert_eq!(
            get("scope"),
            Some(
                "https://www.googleapis.com/auth/gmail.readonly https://www.googleapis.com/auth/drive"
            )
        );
    }

    // A bare code is used as typed, spaces around it are trimmed
    #[test]
    fn extracts_a_bare_code() {
        assert_eq!(extract_code("  4/0Aabc  \n"), Ok("4/0Aabc".to_string()));
    }

    // A percent-encoded code is decoded
    #[test]
    fn extracts_a_percent_encoded_code() {
        assert_eq!(extract_code("4%2F0Aabc"), Ok("4/0Aabc".to_string()));
    }

    // A pasted redirect address gives its code parameter
    #[test]
    fn extracts_the_code_from_a_redirect_address() {
        assert_eq!(
            extract_code("http://localhost:3000/?code=4%2F0Aabc&scope=email"),
            Ok("4/0Aabc".to_string())
        );
    }

    // A pasted address without a code is rejected
    #[test]
    fn rejects_an_address_without_code() {
        assert!(extract_code("http://localhost:3000/?error=access_denied").is_err());
    }

    // An empty entry is rejected
    #[test]
    fn rejects_an_empty_entry() {
        assert!(extract_code("  \n").is_err());
    }

    // A long token shows only its first and last four characters
    #[test]
    fn masks_a_long_token() {
        assert_eq!(
            mask("1//0abcdefghijklmnopqrstuvwxyz"),
            "1//0...wxyz (30 chars)"
        );
    }

    // A short token shows nothing of itself
    #[test]
    fn masks_a_short_token_completely() {
        assert_eq!(mask("abcd1234"), "**** (8 chars)");
    }
}
