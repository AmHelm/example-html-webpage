// Backend program that will serve the static meme webpage

//#![allow(unused)]

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use std::net::SocketAddr;

#[tokio::main]
async fn main(){
    // Serve html file in the "meme-original" directory under "/meme-original"
    let serve_dir = ServeDir::new("meme-original").not_found_service(ServeFile::new("meme-original/index.html"));

    // Initialize the router
    let app = Router::new().fallback_service(serve_dir);

    // Network adress: IP-adress + port
    // 0.0.0.0 to be reachable from outside
    // 3000 is the port
    let addr = SocketAddr::from(([0,0,0,0],3000)); 

    // Listener enables servers to connect to browser, claims a port (gets it from SocketAddr)
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener,app).await.unwrap()
}

