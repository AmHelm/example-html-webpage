// Backend program that will serve the meme webpage

#![allow(non_snake_case)]

mod auth_handlers;
mod meme_handlers;
mod users_handlers;

use axum::{Router, middleware::{self}, 
            routing::{get, post}};
use tower_http::services::{ServeDir, 
                            ServeFile};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_cookies::CookieManagerLayer; 
use auth_handlers::{me, auth, login_user, logout_user};
use sqlx::sqlite::{ SqlitePool, 
            SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use meme_handlers::{create_meme_table, get_random_meme, add_meme};

use crate::users_handlers::{create_users_table, register_user};

// Which port we want the website to claim
const PORT: u16 = 3000;

// Name of the directory where frontend is stored and the index.html file to read
const DIR_PATH: &str = "meme-original";
const FILE_NAME: &str = "index.html";

// This was userful for login auth: https://github.com/wpcodevo/rust-axum-jwt-auth/blob/master/src/handler.rs
// and for middleware auth: https://docs.rs/axum/latest/axum/middleware/index.html 

// Shares data across handlers
// Arc: lets multiple owners share the same data
// Mutex: Allows for dafe modification of shared data
#[derive(Clone)]
struct AppState {
    tokens: Arc<Mutex<HashMap<String, String>>>,
    db: SqlitePool,
}

// Initializes a database where memes (and later on user credentials) will be stored
async fn init_db() -> SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite:memes.db")
        .unwrap()
        .create_if_missing(true);

    SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await
        .unwrap()
}

#[tokio::main]
async fn main(){

    let pool = init_db().await;
    create_meme_table(&pool).await;
    create_users_table(&pool).await;
     
    let state = AppState{
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
        .route("/api/logout", post(logout_user))
        .route("/api/register", post(register_user));
        
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
