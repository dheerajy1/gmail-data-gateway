use crate::{
    db::{azure::public_pool, db_error},
    errors::api_error::ApiError,
};

// One active row of the api users table
#[derive(Clone)]
pub struct ApiUser {
    // Api user name, becomes the token subject
    pub user_name: String,

    // Bcrypt hash of the api user password
    pub password_hash: String,

    // Role of the api user, becomes the token role claim
    pub user_role: String,
}

// Looks up an active api user by name, None when the user does not exist or is not active (D41)
pub async fn find_api_user(username: &str) -> Result<Option<ApiUser>, ApiError> {
    let username = username.to_string();

    let mut connection = public_pool().get().await.map_err(db_error)?;

    let row = connection
        .query(
            "SELECT userName, passwordHash, userRole FROM [02tapiUsers] WHERE userName = @P1 AND isActive = 1",
            &[&username],
        )
        .await
        .map_err(db_error)?
        .into_row()
        .await
        .map_err(db_error)?;

    Ok(row.and_then(|row| {
        Some(ApiUser {
            user_name: row.get::<&str, _>(0)?.to_string(),
            password_hash: row.get::<&str, _>(1)?.to_string(),
            user_role: row.get::<&str, _>(2)?.to_string(),
        })
    }))
}
