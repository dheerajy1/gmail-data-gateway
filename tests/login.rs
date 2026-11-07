mod common;

use axum::{
    body::Body,
    http::{Response, StatusCode, header},
};
use common::{auth_headers, body_json, body_text, send, send_with_body};
use jobs_api_gateway::{
    app::router,
    errors::catalog::{bad_request, forbidden},
};

// Content type of a browser form post
const FORM: (&str, &str) = ("content-type", "application/x-www-form-urlencoded");

// Posts a form body to /login with the given extra headers
async fn post_form(headers: &[(&str, &str)], body: &str) -> Response<Body> {
    let mut all = vec![FORM];

    all.extend_from_slice(headers);

    send_with_body(router(), "POST", "/login", &all, body).await
}

// GET /login returns the login form as html without any credentials
#[tokio::test]
async fn login_page_returns_200_with_form() {
    let response = send(router(), "GET", "/login", &[]).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers()[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .contains("text/html")
    );

    let body = body_text(response).await;

    assert!(body.contains("<form"));
    assert!(body.contains("name=\"username\""));
    assert!(body.contains("name=\"password\""));
    assert!(body.contains("SIGN IN"));
    assert!(body.contains("Gmail data gateway API"));
}

// GET /login sends a logged in browser home
#[tokio::test]
async fn login_page_redirects_a_logged_in_browser() {
    let response = send(router(), "GET", "/login", &[("cookie", "auth=ok")]).await;

    assert_eq!(response.status(), StatusCode::FOUND);
    assert_eq!(response.headers()[header::LOCATION], "/");
}

// POST /login is for browsers only, valid client headers do not replace the browser Accept header
#[tokio::test]
async fn login_post_without_html_accept_returns_403() {
    let response = post_form(&auth_headers(), "username=admin&password=secret1").await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = body_json(response).await;

    assert_eq!(body["code"], forbidden::ACCESS_DENIED.code.as_str());
    assert_eq!(body["error"], forbidden::ACCESS_DENIED.error);
}

// POST /login with a password shorter than 6 characters returns 400 before the database is used
#[tokio::test]
async fn login_post_with_short_password_returns_400() {
    let response = post_form(&[("accept", "text/html")], "username=admin&password=123").await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = body_json(response).await;

    assert_eq!(body["error"], bad_request::INVALID_INPUT.error);
}

// POST /login with a body that is not a form returns 400 before the database is used
#[tokio::test]
async fn login_post_with_invalid_body_returns_400() {
    let response = send_with_body(
        router(),
        "POST",
        "/login",
        &[("accept", "text/html")],
        "username=admin",
    )
    .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
