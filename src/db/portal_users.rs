use crate::{
    db::{azure::public_pool, db_error},
    errors::api_error::ApiError,
};

// Looks up the bcrypt password hash of an active portal user, None when the user does not exist or is not active (D41)
pub async fn get_password_hash(username: &str) -> Result<Option<String>, ApiError> {
    let username = username.to_string();

    let mut connection = public_pool().get().await.map_err(db_error)?;

    let row = connection
        .query(
            "SELECT passwordHash FROM [01tportalUsers] WHERE userName = @P1 AND isActive = 1",
            &[&username],
        )
        .await
        .map_err(db_error)?
        .into_row()
        .await
        .map_err(db_error)?;

    Ok(row.and_then(|row| row.get::<&str, _>(0).map(str::to_string)))
}
