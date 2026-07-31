#![allow(non_snake_case)]

use axum::{Json, extract::State, 
            http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, sqlite::SqlitePool};

use crate::AppState;

// Struct for the incoming new memes from the frontend
// serde deserializes the code (unwraps the Json format)
#[derive(Deserialize)]
pub struct NewMeme {
    text: String,
}

// Struct for stored memes
#[derive(Serialize, FromRow)]
pub struct Meme {
    id: i64,
    text: String,
}

// Initializes a new table in the database to store memes in
pub async fn create_meme_table(pool: &SqlitePool) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS memes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL UNIQUE
        )",
    )
    .execute(pool)
    .await
    .unwrap();
}

// Requests the database for a random meme and serves it
pub async fn get_random_meme(State(state): State<AppState>) -> Result<Json<Meme>,StatusCode> {
    let one_meme = sqlx::query_as::<_,Meme>(
        "SELECT id, text FROM memes ORDER BY RANDOM() LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {eprintln!("db read failed: {e}"); 
    StatusCode::INTERNAL_SERVER_ERROR})?;

    match one_meme{
        Some(meme) => Ok(Json(meme)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// This function checks if the submission is valid (no empty submissions, max limit, no line-breaks and no duplicates)
// This was helpful here: https://dev.to/syeedmdtalha/error-handling-in-axum-31a2 
fn validate_meme(text: &str) -> Result<(), (StatusCode, String)> {

    let max_length: usize = 200;

    // Checks if the submission is empty, is over a max limit of characters, contains a linebreak or is a duplicate
    if text.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Submission cannot be empty!".to_string()));
    }
    
    if text.chars().count() > max_length {
        return Err((StatusCode::BAD_REQUEST, "Submission cannot be more than 200 characters!".to_string()));
    }

    if text.contains("\n") || text.contains("\r") {
        return Err((StatusCode::BAD_REQUEST, "Submission cannot have line-breaks!".to_string()));
    }

    Ok(())
    
}
// Unit tests for validate_meme()
#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn validate_meme__should_return_bad_request_on_empty_strings(){
        let (status, _) = validate_meme("").unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_meme__should_return_bad_request_on_long_strings(){
        let too_long_text = "a".repeat(201);
        let (status, _) = validate_meme(&too_long_text).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_meme__should_return_bad_request_on_strings_containing_linebreaks(){
        let (status, _) = validate_meme("not\nallowed").unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_meme__should_return_ok_on_a_valid_meme(){
        assert!(validate_meme("Testing, testing...").is_ok());
    }
}

// Checks if the submitted meme is valid and adds to the database
pub async fn add_meme(
    State(state): State<AppState>,
    Json(payload): Json<NewMeme>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    
    let text: String = payload.text.trim().to_string();
    
    validate_meme(&text)?;

    let result = sqlx::query("INSERT INTO memes (text) VALUES (?)")
    .bind(&text)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Ok((StatusCode::CREATED, "Meme submitted!".to_string())),
        Err(e) => {
            if e.as_database_error().is_some_and(|db_err| db_err.is_unique_violation()) {
                Err((StatusCode::CONFLICT, "This submission already exists!".to_string()))
            }
            else {
                eprintln!("db insert failed: {e}");
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Could not save meme!".to_string()))
            }
        }
    }
}