use axum::{
    Form, Router,
    extract::rejection::FormRejection,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use validator::Validate;

use super::render::render_with_props;
use crate::{
    components::login::{Login, LoginProps},
    db::portal_users::get_password_hash,
    errors::{
        api_error::ApiError,
        catalog::{bad_request, forbidden, unauthorized},
    },
    headers::wants_html,
    middleware::server_gate::has_auth_cookie,
    schemas::auth::LoginForm,
    security::verify_password,
};

// Plain auth cookie like Elysia, valid for one day and only sent over https
const AUTH_COOKIE: &str = "auth=ok; HttpOnly; SameSite=Strict; Max-Age=86400; Path=/; Secure";

// Routes of the Login page
pub fn router() -> Router {
    Router::new().route("/login", get(login).post(login_submit))
}

// GET /login, a logged in browser is sent home instead of seeing the form
async fn login(headers: HeaderMap) -> Response {
    if has_auth_cookie(&headers) {
        return (StatusCode::FOUND, [(header::LOCATION, "/")]).into_response();
    }

    render_with_props(
        Login,
        LoginProps {
            title: "Login | Gmail data gateway API".to_string(),
        },
    )
    .into_response()
}

// POST /login, browser form submit only
async fn login_submit(
    headers: HeaderMap,
    form: Result<Form<LoginForm>, FormRejection>,
) -> Result<Response, ApiError> {
    // Only browsers submitting the form are allowed
    if !wants_html(&headers) {
        return Err(ApiError::from_entry(forbidden::ACCESS_DENIED));
    }

    // A body that is not a valid form returns 400
    let Ok(Form(form)) = form else {
        return Err(ApiError::from_entry(bad_request::INVALID_INPUT));
    };

    // A password shorter than the minimum returns 400
    if form.validate().is_err() {
        return Err(ApiError::from_entry(bad_request::INVALID_INPUT));
    }

    let hash = get_password_hash(&form.username).await?;

    // Always runs one bcrypt check, a missing user is checked against a dummy hash (D42)
    let is_valid = verify_password(&form.password, hash.as_deref()).await?;

    // An unknown user, an inactive user and a wrong password get the same 401 (D42)
    if !is_valid {
        return Err(ApiError::from_entry(unauthorized::INVALID_CREDENTIALS));
    }

    // Success sets the cookie and redirects home
    Ok((
        StatusCode::FOUND,
        [(header::LOCATION, "/"), (header::SET_COOKIE, AUTH_COOKIE)],
    )
        .into_response())
}
