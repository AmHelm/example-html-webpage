// Backend program that will serve the meme webpage

#![allow(non_snake_case)]

mod auth_handlers;

use axum::{Json, Router, extract::State, 
            http::StatusCode, middleware::self, 
            routing::{get, post}};
use tower_http::services::{ServeDir, 
                            ServeFile};
use std::collections::HashMap;
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_cookies::CookieManagerLayer; 
use auth_handlers::{hash_password, me, 
                    auth, login_user, logout_user};
use sqlx::{FromRow, sqlite::{ SqlitePool, 
            SqliteConnectOptions, SqlitePoolOptions}};
use std::str::FromStr;

// Which port we want the website to claim
const PORT: u16 = 3000;

// Name of the directory where frontend is stored and the index.html file to read
const DIR_PATH: &str = "meme-original";
const FILE_NAME: &str = "index.html";

// Struct for the incoming new memes from the frontend
// serde deserializes the code (unwraps the Json format)
#[derive(Deserialize)]
struct NewMeme {
    text: String,
}

#[derive(Serialize, FromRow)]
struct Meme {
    id: i64,
    text: String,
}

// This was userful for login auth: https://github.com/wpcodevo/rust-axum-jwt-auth/blob/master/src/handler.rs
// and for middleware auth: https://docs.rs/axum/latest/axum/middleware/index.html 

// Shares data across handlers
// Arc: lets multiple owners share the same data
// Mutex: Allows for dafe modification of shared data
#[derive(Clone)]
struct AppState {
    users: Arc<HashMap<String, String>>,
    tokens: Arc<Mutex<HashMap<String, String>>>,
    db: SqlitePool,
}

async fn init_db() -> SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite:memes.db")
        .unwrap()
        .create_if_missing(true);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .unwrap()
}

async fn create_table(pool: &SqlitePool) {
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

async fn get_random_meme(State(state): State<AppState>) -> Result<Json<Meme>,StatusCode> {
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

async fn add_meme(
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

#[tokio::main]
async fn main(){

    let pool = init_db().await;
    create_table(&pool).await;

    let mut users = HashMap::new();
    users.insert("amanda".to_string(), hash_password("1234"));

     
    let state = AppState{
        users: Arc::new(users),
        tokens: Arc::new(Mutex::new(HashMap::new())),
        db: pool,
    };
    

    let file_path = format!("{DIR_PATH}/{FILE_NAME}");

    // Serve html file in the DIR_NAME directory
    let serve_dir = ServeDir::new(DIR_PATH)
                                                    .not_found_service(ServeFile::new(file_path));

    // Initialize the router
    // Reachable before entering credentials
    let public = Router::new()
        .route("/api/login", post(login_user))
        .route("/api/logout", post(logout_user));
        
    // Reachable after logging in
    let protected = Router::new()
        .route("/api/add_meme", post(add_meme))
        .route("/api/get_random_meme", get(get_random_meme))
        .route("/api/me", get(me))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth));

    // Merges the routes
    let app = Router::new()
        .merge(public)
        .merge(protected)
        .fallback_service(serve_dir)
        .layer(CookieManagerLayer::new())  
        .with_state(state);

    // Network adress: IP-adress + port
    // 0.0.0.0 to be reachable from outside
    let addr = SocketAddr::from(([0,0,0,0],PORT)); 

    // Listener enables servers to connect to browser, claims a port (gets it from SocketAddr)
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener,app).await.unwrap()
}
