mod common;

use axum::{
    body::Body,
    http::{Response, StatusCode},
};
use common::{TEST_JWT_SECRET, auth_headers, body_json, send, send_with_body, setup_env};
use jobs_api_gateway::{
    app::router,
    errors::catalog::bad_request,
    features::v1::auth::jwt::{ISSUER, sign_token, verify_token},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

// Posts a json body to the API login route with the given headers
async fn post_login(headers: &[(&str, &str)], body: &str) -> Response<Body> {
    let mut all = vec![("content-type", "application/json")];

    all.extend_from_slice(headers);

    send_with_body(router(), "POST", "/api/v1/auth/login", &all, body).await
}

// POST /api/v1/auth/login without client headers returns 401, the cookie lets the request past the gate
#[tokio::test]
async fn api_login_without_headers_returns_401() {
    let response = post_login(
        &[("cookie", "auth=ok")],
        r#"{"username":"admin","password":"secret1"}"#,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = body_json(response).await;

    assert_eq!(body["code"], "UNAUTHORIZED");
}

// POST /api/v1/auth/login with a body that is not json returns 400
#[tokio::test]
async fn api_login_with_invalid_json_returns_400() {
    let response = post_login(&auth_headers(), "not json").await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body_json(response).await;

    assert_eq!(body["error"], bad_request::INVALID_INPUT.error);
}

// POST /api/v1/auth/login with a short password returns 400
#[tokio::test]
async fn api_login_with_short_password_returns_400() {
    let response = post_login(&auth_headers(), r#"{"username":"admin","password":"123"}"#).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// POST /api/v1/auth/login with an empty username returns 400
#[tokio::test]
async fn api_login_with_empty_username_returns_400() {
    let response = post_login(&auth_headers(), r#"{"username":"","password":"secret1"}"#).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// GET /api/v1/auth/login returns 405, the route only accepts POST
#[tokio::test]
async fn api_login_rejects_get() {
    let response = send(router(), "GET", "/api/v1/auth/login", &auth_headers()).await;

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

// A signed token carries the subject, role, issuer and issue time, and expires in one day
#[test]
fn signed_token_has_expected_claims() {
    setup_env();

    let data = sign_token("admin", "admin").unwrap();

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[ISSUER]);

    let decoded = decode::<jobs_api_gateway::features::v1::auth::model::JwtClaims>(
        &data.token,
        &DecodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
        &validation,
    )
    .unwrap();

    assert_eq!(decoded.claims.sub, "admin");
    assert_eq!(decoded.claims.role, "admin");
    assert_eq!(decoded.claims.iss, "jobs-api-gateway");
    assert!(decoded.claims.iat > 0);
    assert_eq!(decoded.claims.exp - decoded.claims.iat, 86_400);
    assert!(data.expires_at.ends_with('Z'));
}

// A signed token passes the verification used by the Auth extractor
#[test]
fn signed_token_passes_verification() {
    setup_env();

    let data = sign_token("api_user", "user").unwrap();
    let claims = verify_token(&data.token).unwrap();

    assert_eq!(claims.sub, "api_user");
    assert_eq!(claims.role, "user");
}
