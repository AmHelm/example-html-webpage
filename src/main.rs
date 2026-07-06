// Backend program that will serve the static meme webpage

#![allow(unused)]

use axum::{Router, 
           routing::get, 
           Json};
use tower_http::services::{ServeDir, 
                           ServeFile};
use std::net::SocketAddr;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::io;
use rand::{Rng, seq::IndexedRandom};

// Use reqwest to make GET requests from webpage?

// Which port we want the website to claim
const PORT: u16 = 3000;

// Name of the directory where frontend is stored and the index.html file to read
const DIR_PATH: &str = "meme-original";
const FILE_NAME: &str = "index.html";

// The name of the text file we want to read
const MEME_TEXTS: &str = "meme-texts.md";


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
    let mut random_meme = memes.choose(&mut rng).unwrap().to_string();

    // Wrap the text files into Json format
    Json(random_meme)
}


#[tokio::main]
async fn main(){

    let file_path = format!("{DIR_PATH}/{FILE_NAME}");

    // Serve html file in the DIR_NAME directory
    let serve_dir = ServeDir::new(DIR_PATH)
                                                    .not_found_service(ServeFile::new(file_path));

    // Initialize the router
    let app = Router::new()
                      .route("/api/get_random_meme", get(get_random_meme))
                      .fallback_service(serve_dir);

    // Network adress: IP-adress + port
    // 0.0.0.0 to be reachable from outside
    let addr = SocketAddr::from(([0,0,0,0],PORT)); 

    // Listener enables servers to connect to browser, claims a port (gets it from SocketAddr)
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener,app).await.unwrap()
}

