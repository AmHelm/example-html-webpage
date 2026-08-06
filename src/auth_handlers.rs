use axum::{Json, extract::{Extension, Request, State}, 
            http::StatusCode, middleware::Next, 
            response::Response};
use rand::{distr::Alphanumeric, RngExt};
use argon2::{Argon2, PasswordHash, 
                PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use tower_cookies::{Cookie, Cookies}; 

use crate::{AppState, users_handlers::get_user_from_username};

#[derive(serde::Deserialize)]
pub struct UserCredentials{
    username: String,
    password: String,
}

// Handler for salting and then hashing password
// Salting: random chunk of data that is added to a password before hashing it 
pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap().to_string()
}

// Handler for verifying password
// Checks that the stored hash is the same as the submitted password
pub fn is_valid(password: &str, stored_hash: &str) -> bool {
    match PasswordHash::new(stored_hash) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok(),
        Err(_) => false,
    }
}

// Frontend uses this to check if the user is logged in
pub async fn me(Extension(user): Extension<String>) -> Json<String> {
    Json(user)   
}

// Useful for cookies: https://github.com/imbolc/tower-cookies/tree/main

// Checks if user token matches stored tokens
pub async fn auth(
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
pub async fn login_user(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(body): Json<UserCredentials>) -> Result<StatusCode, (StatusCode, String)> {

    // Message to use if login details are incorrect
    let error_invalid: String = "Invalid username or password".to_string();
    let error_fail: String = "Login failed".to_string();

    // Looks up the stores password for this username
    let user = get_user_from_username(&state.db, &body.username)
        .await
        .map_err(|e| {eprintln!("db read failed: {e}"); (StatusCode::INTERNAL_SERVER_ERROR, error_fail)})?
        .ok_or((StatusCode::UNAUTHORIZED, error_invalid.clone()))?;

    // Passes the submitted password and stored hash to handler for a validity check 
    if !is_valid(&body.password, &user.password_hash) {
        return Err((StatusCode::UNAUTHORIZED, error_invalid));
    }

    // Generate a random token
    let token: String = rand::rng().sample_iter(&Alphanumeric).take(50).map(char::from).collect();
    
    // Inserts token into AppState struct
    state.tokens.lock().unwrap().insert(token.clone(), user.username.clone());

    // Store token as cookie
    let mut cookie = Cookie::new("auth_token", token);
    cookie.set_path("/");
    cookies.add(cookie);

    Ok(StatusCode::OK)
}

// Handler to log out user
pub async fn logout_user(State(state): State<AppState>, cookies: Cookies) -> StatusCode {
    // Read token from "auth_token" cookie
    // If there is a cookie, remove it from AppState
    if let Some(cookie) = cookies.get("auth_token") {
        state.tokens.lock().unwrap().remove(cookie.value());
    }
    // Remove cookie from browser
    cookies.remove(Cookie::new("auth_token", ""));
    StatusCode::OK
}