use super::{
    jwt::sign_token,
    model::{LoginData, LoginRequest},
};
use crate::{
    db::api_users::find_api_user,
    errors::{api_error::ApiError, catalog::unauthorized},
    security::verify_password,
};

// Checks the api user credentials and returns a signed token
pub async fn login(request: LoginRequest) -> Result<LoginData, ApiError> {
    let user = find_api_user(&request.username).await?;

    // Always runs one bcrypt check, a missing user is checked against a dummy hash (D42)
    let is_valid = verify_password(
        &request.password,
        user.as_ref().map(|user| user.password_hash.as_str()),
    )
    .await?;

    match user {
        Some(user) if is_valid => sign_token(&user.user_name, &user.user_role),
        // An unknown user, an inactive user and a wrong password get the same 401 (D42)
        _ => Err(ApiError::from_entry(unauthorized::INVALID_CREDENTIALS)),
    }
}
