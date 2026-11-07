mod common;

use axum::{Router, body::Body, http::Request, http::StatusCode, routing::get};
use common::{auth_headers, body_json, send};
use jobs_api_gateway::{app::router, errors::panic::handle_panic};
use tower::ServiceExt;
use tower_http::catch_panic::CatchPanicLayer;

// A handler that always panics, used to test the panic layer
async fn boom() -> String {
    panic!("test panic");
}

// An unknown path returns the global 404 error shape
#[tokio::test]
async fn unknown_path_returns_404_error_shape() {
    let response = send(router(), "GET", "/api/v1/dummy", &auth_headers()).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = body_json(response).await;

    assert_eq!(body["success"], false);
    assert_eq!(body["statusCode"], 404);
    assert_eq!(body["code"], "NOT_FOUND");
    assert!(body["message"].is_string());
    assert!(body["error"].is_string());
}

// A known path with the wrong method returns the global 405 error shape
#[tokio::test]
async fn wrong_method_returns_405_error_shape() {
    let response = send(router(), "POST", "/api/v1/health", &auth_headers()).await;

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

    let body = body_json(response).await;

    assert_eq!(body["success"], false);
    assert_eq!(body["statusCode"], 405);
    assert_eq!(body["code"], "METHOD_NOT_SUPPORTED");
}

// A panic inside a handler returns the global 500 error shape and the router keeps working
#[tokio::test]
async fn panic_returns_500_error_shape() {
    let app = Router::new()
        .route("/boom", get(boom))
        .layer(CatchPanicLayer::custom(handle_panic));

    let request = Request::builder().uri("/boom").body(Body::empty()).unwrap();
    let response = app.clone().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = body_json(response).await;

    assert_eq!(body["success"], false);
    assert_eq!(body["statusCode"], 500);
    assert_eq!(body["code"], "INTERNAL_SERVER_ERROR");

    // The same router still answers after the panic
    let request = Request::builder()
        .uri("/missing")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
