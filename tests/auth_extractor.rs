mod common;

use axum::{
    Json, Router,
    body::Body,
    http::{Response, StatusCode},
    routing::get,
};
use chrono::Utc;
use common::{TEST_JWT_SECRET, auth_headers, body_json, send, setup_env};
use jobs_api_gateway::{
    errors::catalog::{CatalogEntry, bad_request, unauthorized},
    features::v1::auth::{jwt::ISSUER, model::JwtClaims},
    middleware::auth::Auth,
};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde_json::{Value, json};

// Handler that only runs when the Auth extractor accepts the request
async fn protected(auth: Auth) -> Json<Value> {
    Json(json!({
        "username": auth.username,
        "role": format!("{:?}", auth.role),
    }))
}

// Sends a request to the test route with the given headers
async fn call(headers: &[(&str, &str)]) -> Response<Body> {
    let router = Router::new().route("/protected", get(protected));

    send(router, "GET", "/protected", headers).await
}

// Client headers followed by an Authorization header with the given value
fn with_authorization(authorization: &str) -> Vec<(&str, &str)> {
    let mut headers: Vec<(&str, &str)> = auth_headers().to_vec();

    headers.push(("authorization", authorization));

    headers
}

// Seconds since the unix epoch, used to build fresh and expired tokens
fn now() -> u64 {
    Utc::now().timestamp().unsigned_abs()
}

// Signs a token with the given secret, claims and expiry
fn token_with(secret: &str, sub: &str, role: &str, iss: &str, exp: u64) -> String {
    let claims = JwtClaims {
        sub: sub.to_string(),
        role: role.to_string(),
        iss: iss.to_string(),
        iat: now(),
        exp,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

// A token the extractor accepts, for the given subject and role
fn valid_token(sub: &str, role: &str) -> String {
    token_with(TEST_JWT_SECRET, sub, role, ISSUER, now() + 3600)
}

// The response has the status, code and error text of the catalogue entry
async fn assert_entry(response: Response<Body>, status: StatusCode, entry: CatalogEntry) {
    assert_eq!(response.status(), status);

    let body = body_json(response).await;

    assert_eq!(body["success"], false);
    assert_eq!(body["statusCode"], status.as_u16());
    assert_eq!(body["code"], entry.code.as_str());
    assert_eq!(body["error"], entry.error);
}

// A valid admin token reaches the handler with the subject and role
#[tokio::test]
async fn valid_admin_token_reaches_handler() {
    setup_env();

    let token = valid_token("api_user", "admin");
    let response = call(&with_authorization(&format!("Bearer {token}"))).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;

    assert_eq!(body["username"], "api_user");
    assert_eq!(body["role"], "Admin");
}

// A valid user token reaches the handler with the user role
#[tokio::test]
async fn valid_user_token_reaches_handler() {
    setup_env();

    let token = valid_token("api_user", "user");
    let response = call(&with_authorization(&format!("Bearer {token}"))).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = body_json(response).await;

    assert_eq!(body["role"], "User");
}

// Missing client headers return 401 MISSING_AUTH_HEADERS
#[tokio::test]
async fn missing_client_headers_return_401() {
    let response = call(&[]).await;

    assert_entry(
        response,
        StatusCode::UNAUTHORIZED,
        unauthorized::MISSING_AUTH_HEADERS,
    )
    .await;
}

// A wrong client secret returns 400 INVALID_HEADER_VALUES
#[tokio::test]
async fn wrong_client_secret_returns_400() {
    let response = call(&[("x-client-id", "test-client"), ("x-client-secret", "wrong")]).await;

    assert_entry(
        response,
        StatusCode::BAD_REQUEST,
        bad_request::INVALID_HEADER_VALUES,
    )
    .await;
}

// A wrong client id returns 400 INVALID_HEADER_VALUES
#[tokio::test]
async fn wrong_client_id_returns_400() {
    let response = call(&[("x-client-id", "wrong"), ("x-client-secret", "test-secret")]).await;

    assert_entry(
        response,
        StatusCode::BAD_REQUEST,
        bad_request::INVALID_HEADER_VALUES,
    )
    .await;
}

// A client id longer than 30 characters returns 400 INVALID_HEADER_VALUES
#[tokio::test]
async fn too_long_client_id_returns_400() {
    let long_id = "a".repeat(31);
    let response = call(&[
        ("x-client-id", long_id.as_str()),
        ("x-client-secret", "test-secret"),
    ])
    .await;

    assert_entry(
        response,
        StatusCode::BAD_REQUEST,
        bad_request::INVALID_HEADER_VALUES,
    )
    .await;
}

// An empty client id returns 400 INVALID_HEADER_VALUES
#[tokio::test]
async fn empty_client_id_returns_400() {
    let response = call(&[("x-client-id", ""), ("x-client-secret", "test-secret")]).await;

    assert_entry(
        response,
        StatusCode::BAD_REQUEST,
        bad_request::INVALID_HEADER_VALUES,
    )
    .await;
}

// A missing Authorization header returns 401 MISSING_AUTHORIZATION_TOKEN
#[tokio::test]
async fn missing_authorization_returns_401() {
    let response = call(&auth_headers()).await;

    assert_entry(
        response,
        StatusCode::UNAUTHORIZED,
        unauthorized::MISSING_AUTHORIZATION_TOKEN,
    )
    .await;
}

// A non-bearer scheme returns 401 MISSING_AUTHORIZATION_TOKEN
#[tokio::test]
async fn non_bearer_scheme_returns_401() {
    let response = call(&with_authorization("Basic abc123")).await;

    assert_entry(
        response,
        StatusCode::UNAUTHORIZED,
        unauthorized::MISSING_AUTHORIZATION_TOKEN,
    )
    .await;
}

// A bearer scheme with an empty token returns 401 MISSING_AUTHORIZATION_TOKEN
#[tokio::test]
async fn empty_bearer_token_returns_401() {
    let response = call(&with_authorization("Bearer ")).await;

    assert_entry(
        response,
        StatusCode::UNAUTHORIZED,
        unauthorized::MISSING_AUTHORIZATION_TOKEN,
    )
    .await;
}

// A token that is not a jwt returns 401 INVALID_TOKEN
#[tokio::test]
async fn garbage_token_returns_401() {
    let response = call(&with_authorization("Bearer not-a-token")).await;

    assert_entry(
        response,
        StatusCode::UNAUTHORIZED,
        unauthorized::INVALID_TOKEN,
    )
    .await;
}

// A token signed with another secret returns 401 INVALID_TOKEN
#[tokio::test]
async fn token_with_wrong_secret_returns_401() {
    let token = token_with("another-secret", "api_user", "admin", ISSUER, now() + 3600);
    let response = call(&with_authorization(&format!("Bearer {token}"))).await;

    assert_entry(
        response,
        StatusCode::UNAUTHORIZED,
        unauthorized::INVALID_TOKEN,
    )
    .await;
}

// A token from another issuer returns 401 INVALID_TOKEN
#[tokio::test]
async fn token_with_wrong_issuer_returns_401() {
    let token = token_with(
        TEST_JWT_SECRET,
        "api_user",
        "admin",
        "another-api",
        now() + 3600,
    );
    let response = call(&with_authorization(&format!("Bearer {token}"))).await;

    assert_entry(
        response,
        StatusCode::UNAUTHORIZED,
        unauthorized::INVALID_TOKEN,
    )
    .await;
}

// An expired token returns 401 EXPIRED_TOKEN (D38)
#[tokio::test]
async fn expired_token_returns_401() {
    let token = token_with(TEST_JWT_SECRET, "api_user", "admin", ISSUER, 1);
    let response = call(&with_authorization(&format!("Bearer {token}"))).await;

    assert_entry(
        response,
        StatusCode::UNAUTHORIZED,
        unauthorized::EXPIRED_TOKEN,
    )
    .await;
}

// A token with an unknown role returns 401 MISSING_SUB_OR_ROLE
#[tokio::test]
async fn token_with_unknown_role_returns_401() {
    let token = valid_token("api_user", "root");
    let response = call(&with_authorization(&format!("Bearer {token}"))).await;

    assert_entry(
        response,
        StatusCode::UNAUTHORIZED,
        unauthorized::MISSING_SUB_OR_ROLE,
    )
    .await;
}

// A token with an empty subject returns 401 MISSING_SUB_OR_ROLE
#[tokio::test]
async fn token_with_empty_subject_returns_401() {
    let token = valid_token("", "admin");
    let response = call(&with_authorization(&format!("Bearer {token}"))).await;

    assert_entry(
        response,
        StatusCode::UNAUTHORIZED,
        unauthorized::MISSING_SUB_OR_ROLE,
    )
    .await;
}
