// Backend program that will serve the static meme webpage

//#![allow(unused)]

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use std::net::SocketAddr;

// Which port we want the website to claim
const PORT: u16 = 3000;

// Name of the directory where frontend is stored and the index.html file to read
const DIR_PATH: &str = "meme-original";
const FILE_NAME: &str = "index.html";

#[tokio::main]
async fn main(){

    let file_path = format!("{DIR_PATH}/{FILE_NAME}");

    // Serve html file in the DIR_NAME directory
    let serve_dir = ServeDir::new(DIR_PATH).not_found_service(ServeFile::new(file_path));

    // Initialize the router
    let app = Router::new().fallback_service(serve_dir);

    // Network adress: IP-adress + port
    // 0.0.0.0 to be reachable from outside
    let addr = SocketAddr::from(([0,0,0,0],PORT)); 

    // Listener enables servers to connect to browser, claims a port (gets it from SocketAddr)
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener,app).await.unwrap()
}

