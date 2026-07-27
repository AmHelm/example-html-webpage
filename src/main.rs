// Backend program that will serve the meme webpage

#![allow(unused)]
#![allow(non_snake_case)]

use axum::{Json, Router, extract::{Extension, Request, State}, 
            http::StatusCode, middleware::{self, Next}, 
            response::{IntoResponse, Response}, 
            routing::{Route, get, post}};
use tower_http::services::{ServeDir, 
                            ServeFile};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::fs::{File, 
                OpenOptions};
use std::io::{BufReader, 
                BufRead, Write};
use std::io;
use rand::{Rng, distr::Alphanumeric, 
            seq::IndexedRandom, RngExt};
use serde::Deserialize;
use argon2::{Argon2, PasswordHash, 
                PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use std::sync::{Arc, Mutex};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies}; 


// Which port we want the website to claim
const PORT: u16 = 3000;

// Name of the directory where frontend is stored and the index.html file to read
const DIR_PATH: &str = "meme-original";
const FILE_NAME: &str = "index.html";

// The name of the text file we want to read
const MEME_TEXTS: &str = "meme-texts.md";

// Struct for the incoming new memes from the frontend
// serde deserializes the code (unwraps the Json format)
#[derive(serde::Deserialize)]
struct NewMeme {
    text: String,
}

// This was userful for login auth: https://github.com/wpcodevo/rust-axum-jwt-auth/blob/master/src/handler.rs
// and for middleware auth: https://docs.rs/axum/latest/axum/middleware/index.html 

#[derive(serde::Deserialize)]
struct UserCredentials{
    username: String,
    password: String,
}

// Shares data across handlers
// Arc: lets multiple owners share the same data
// Mutex: Allows for dafe modification of shared data
#[derive(Clone)]
struct AppState {
    users: Arc<HashMap<String, String>>,
    tokens: Arc<Mutex<HashMap<String, String>>>,
}

// Handler for salting and then hashing password
// Salting: random chunk of data that is added to a password before hashing it 
fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap().to_string()
}

// Handler for verifying password
// Checks that the stored hash is the same as the submitted password
fn is_valid(password: &str, stored_hash: &str) -> bool {
    match PasswordHash::new(stored_hash) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok(),
        Err(_) => false,
    }
}

// Frontend uses this to check if the user is logged in
async fn me(Extension(user): Extension<String>) -> Json<String> {
    Json(user)   
}

// Useful for cookies: https://github.com/imbolc/tower-cookies/tree/main

// Checks if user token matches stored tokens
async fn auth(
    State(state): State<AppState>,
    cookies: Cookies,
    mut request: Request,     
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    // 
    let token = cookies
        .get("auth_token")
        .map(|c| c.value().to_string())
        .ok_or((StatusCode::UNAUTHORIZED, "Not logged in".to_string()))?;

    // Retrieves the username associated with this token
    let username = state.tokens.lock().unwrap().get(&token).cloned();

    // Checks if token matches a stored user token, either accepts or rejects letting the user through
    match username {
        Some(user) => {
            request.extensions_mut().insert(user); 
            Ok(next.run(request).await)
        }
        None => Err((StatusCode::UNAUTHORIZED, "Invalid token".to_string())), 
    }
}

// Login function
async fn login_user(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(body): Json<UserCredentials>) -> Result<StatusCode, (StatusCode, String)> {

    // Message to use if login details are incorrect
    let error_response: String = "Invalid username or password".to_string();

    // Looks up the stores password for this username
    let stored_hash = state.users.get(&body.username).ok_or((StatusCode::UNAUTHORIZED, error_response.clone()))?;

    // Passes the submitted password and stored hash to handler for a validity check 
    if !is_valid(&body.password, stored_hash) {
        return Err((StatusCode::UNAUTHORIZED, error_response));
    }

    // Generate a random token
    let token: String = rand::rng().sample_iter(&Alphanumeric).take(50).map(char::from).collect();
    
    // Inserts token into AppState struct
    state.tokens.lock().unwrap().insert(token.clone(), body.username.clone());

    // Store token as cookie
    let mut cookie = Cookie::new("auth_token", token);
    cookie.set_path("/");
    cookies.add(cookie);

    Ok(StatusCode::OK)
}

// Handler to log out user
async fn logout_user(State(state): State<AppState>, cookies: Cookies) -> StatusCode {
    // Read token from "auth_token" cookie
    // If there is a cookie, remove it from AppState
    if let Some(cookie) = cookies.get("auth_token") {
        state.tokens.lock().unwrap().remove(cookie.value());
    }
    // Remove cookie from browser
    cookies.remove(Cookie::new("auth_token", ""));
    StatusCode::OK
}

// Function that reads a file containing strings and returns a list
fn read_memes_from_file() -> io::Result<Vec<String>> {

    // Read the file
    let file = File::open(MEME_TEXTS).unwrap();

    // BufReader lets us read the file line by line
    let reader = BufReader::new(file);

    // Make a vector to contain the text lines
    let mut lines: Vec<String>   = Vec::new();

    // Add each text line into a vector
    for line in reader.lines() {
        let line = line?;
        lines.push(line);
    }

    // If no error, return the vector containing all the text lines
    Ok(lines)
}

// Gets the meme texts and randomizes one of them
// Sends off the meme string in Json format
async fn get_random_meme() -> Json<String> {
    
    let memes = read_memes_from_file().unwrap();

    // Randomizer
    let mut rng = rand::rng();
    let random_meme = memes.choose(&mut rng).unwrap().to_string();

    // Wrap the text files into Json format
    Json(random_meme)
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

    let memes = read_memes_from_file().unwrap();
    // This was helpful here: https://sts10.github.io/2019/06/06/is-all-equal-function.html
    if memes.iter().any(|meme| meme == text){ // Checks for exact duplicates, it will be case sensitive
        return Err((StatusCode::CONFLICT, "This submission already exists!".to_string()));    
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

// This function reads the new meme from the frontend, unwraps the Json format and sends the string to new_meme_to_file
async fn add_meme(Json(payload): Json<NewMeme>) -> (StatusCode, String) {

    let text: &str = payload.text.trim();

    // If the meme isn't valid, don't add it to the file and raise error
    if let Err(error) = validate_meme(text){
        return error;

    }

    // If the meme is valid, add it to the file
    new_meme_to_file(text).unwrap();
    (StatusCode::CREATED, "Meme submitted!".to_string())
}

// Adds the submitted meme text to the meme-texts.md file
// https://www.programiz.com/rust/file-handling
fn new_meme_to_file(text: &str) -> io::Result<()> {

    // Open a file with append option
    let mut meme_file = OpenOptions::new()
        .append(true)
        .open(MEME_TEXTS)?;

    // Write to a file on a new line
    writeln!(meme_file, "{text}")?;
    Ok(())
}

#[tokio::main]
async fn main(){

    let mut users = HashMap::new();
    users.insert("amanda".to_string(), hash_password("1234"));

     
    let state = AppState{
        users: Arc::new(users),
        tokens: Arc::new(Mutex::new(HashMap::new())),
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
