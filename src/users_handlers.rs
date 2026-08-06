#![allow(non_snake_case)]
#![allow(dead_code)] //For id field in User struct

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize};
use sqlx::{FromRow, sqlite::SqlitePool};

use crate::{AppState, auth_handlers::hash_password};

// Struct for new users for registration
#[derive(Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

// Struct for stored users
#[derive(FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

// Creates a table in the database to store user credentials
pub async fn create_users_table(pool: &SqlitePool){
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username TEXT NOT NULL UNIQUE COLLATE NOCASE,
        password_hash TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await
    .unwrap();
}

// Finds and returns a user by looking up username in the user database table 
pub async fn get_user_from_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<User>, sqlx::Error>{
    
    sqlx::query_as::<_,User>(
        "SELECT id, username, password_hash FROM users WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

// Registers users and adds them to the users database table 
pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<NewUser>,
) -> Result<(StatusCode, String), (StatusCode, String)> {

    let username: String = payload.username.trim().to_string();
    let password: String = payload.password;

    // Check that user credentials are valid before storing
    validate_user_credentials(&username, &password)?;

    // Salts and hashes password 
    let password_hash = hash_password(&password);

    let result = sqlx::query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
    .bind(&username)
    .bind(&password_hash)
    .execute(&state.db)
    .await;

    match result {
        
        Ok(_) => Ok((StatusCode::CREATED, "User registered!".to_string())),

        //Checks for duplicate usernames
        Err(e) => {
            if e.as_database_error().is_some_and(|db_err| db_err.is_unique_violation()) {
                Err((StatusCode::CONFLICT, "This user already exists!".to_string()))
            }
            else {
                eprintln!("db insert failed: {e}");
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Could not save user!".to_string()))
            }
        }
    }
}

// Checks if the username and password that has been entered are valid before registering them
// Checks if the username or password is empty, is over/under a max/min limit of characters
//  contains a linebreak or uses invalid character types
pub fn validate_user_credentials(
    username: &str,
    password: &str,
) -> Result<(),(StatusCode, String)> {

    let max_length_username: usize = 30;

    // Checks if username is empty
    if username.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username cannot be empty!".to_string()));
    }
    
    // Allowed maximum number of characters for a username
    if username.chars().count() > max_length_username {
        let message = format!("Username cannot be more than {max_length_username} characters!");
        return Err((StatusCode::BAD_REQUEST, message));
    }

    // Checks if username contains a line-break
    if username.contains("\n") || username.contains("\r") {
        return Err((StatusCode::BAD_REQUEST, "Username cannot have line-breaks!".to_string()));
    }

    // Allowed characters types 
    if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') {
        return Err((StatusCode::BAD_REQUEST, "Username can only contain letters, numbers, underscores and periods!".to_string()));
    }

    let max_length_password: usize = 100;
    let min_length_password: usize = 8;

    // Checks if password is empty
    if password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Password cannot be empty!".to_string()));
    }
    
    // Allowed maximum/minimum number of characters for a password
    if password.chars().count() > max_length_password || password.chars().count() < min_length_password {
        let message = format!("Password needs to have at least {min_length_password}\
                                        characters and cannot be more than {max_length_password} characters!");
        return Err((StatusCode::BAD_REQUEST, message));
    }

    // Checks if username contains a line-break
    if password.contains("\n") || password.contains("\r") {
        return Err((StatusCode::BAD_REQUEST, "Password cannot have line-breaks!".to_string()));
    }

    Ok(())
}

// Unit tests for validate_user_credentials()
#[cfg(test)]
mod tests{
    use super::*;

    const VALID_USERNAME: &str = "Amanda_1";
    const VALID_PASSWORD: &str = "SuperSecret12345";

    #[test]
    fn validate_user_credentials__should_return_bad_request_on_empty_usernames(){
        let (status, _) = validate_user_credentials("", VALID_PASSWORD).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_user_credentials__should_return_bad_request_on_long_usernames(){
        let too_long_username = "a".repeat(31);
        let (status, _) = validate_user_credentials(&too_long_username, VALID_PASSWORD).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_user_credentials__should_return_bad_request_on_usernames_containing_wrong_characters(){
        let (status, _) = validate_user_credentials("Amanda!", VALID_PASSWORD).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_user_credentials__should_return_bad_request_on_usernames_containing_linebreaks(){
        let (status, _) = validate_user_credentials("not\nallowed", VALID_PASSWORD).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_user_credentials__should_return_bad_request_on_long_passwords(){
        let too_long_password = "a".repeat(101);
        let (status, _) = validate_user_credentials(VALID_USERNAME, &too_long_password).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_user_credentials__should_return_bad_request_on_short_passwords(){
        let (status, _) = validate_user_credentials(VALID_USERNAME, "short1").unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_user_credentials__should_return_bad_request_on_passwords_containing_linebreaks(){
        let (status, _) = validate_user_credentials(VALID_USERNAME, "not\nallowed12345").unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_user_credentials__should_return_ok_on_a_valid_credentials(){
        assert!(validate_user_credentials(VALID_USERNAME,VALID_PASSWORD).is_ok());
    }
}