use super::types::{ApiError, ApiResult, UserRegistration};
use crate::database::types::{Users, RELATIONAL_DATABASE};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use axum::Json;
use crate::database::types::Roles::User;
use serde_json::json; 

#[tracing::instrument]
pub async fn register_user(
    Json(payload): Json<UserRegistration>,
) -> Result<Json<ApiResult>, ApiError> {
    // see if user is already registered (db read)
    // return error if !unique
    // register user if unique
    // insert new
    // return successful response

    let db_connection = RELATIONAL_DATABASE.get().ok_or(ApiError {
        data: String::from("Something has gone horribly wrong"),
    })?;

    let account: Option<Users> =
        sqlx::query_as!(Users, "SELECT * FROM Users WHERE email = ?", &payload.email)
            .fetch_optional(db_connection)
            .await?;

    if let Some(_) = account {
        return Err(ApiError {
            data: String::from("Error - user is already registered"),
        });
    }

    let hashed_pass: String = {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(&payload.password.as_bytes(), &salt)?
            .to_string()
    };

    sqlx::query!(
        "INSERT INTO Users(email, password, role) VALUES (?, ?, ?)",
        &payload.email, hashed_pass, User.to_string()
    )
    .execute(db_connection)
    .await?;

    Ok(Json(ApiResult {
        res: json!(format!("{} was successfully registered as user", payload.email)),
    }))
}
