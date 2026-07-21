// Backend program that will serve the meme webpage

#![allow(unused)]

use axum::{Router, 
            routing::get, 
            routing::post, 
            Json,
            http::StatusCode};
use tower_http::services::{ServeDir, 
                            ServeFile};
use std::net::SocketAddr;
use std::fs::{File, 
                OpenOptions};
use std::io::{BufReader, 
                BufRead, 
                Write};
use std::io;
use rand::{Rng, 
            seq::IndexedRandom};
use serde::Deserialize;

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

use crate::validate_meme;

    #[test]
    fn validate_meme_should_return_bad_request_on_empty_strings(){
        let (status, _) = validate_meme("").unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_meme_should_return_bad_request_on_long_strings(){
        let too_long_text = "a".repeat(201);
        let (status, _) = validate_meme(&too_long_text).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_meme_should_return_bad_request_on_strings_containing_linebreaks(){
        let (status, _) = validate_meme("not\nallowed").unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn validate_meme_should_return_ok_on_a_valid_meme(){
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

    let file_path = format!("{DIR_PATH}/{FILE_NAME}");

    // Serve html file in the DIR_NAME directory
    let serve_dir = ServeDir::new(DIR_PATH)
                                                    .not_found_service(ServeFile::new(file_path));

    // Initialize the router
    let app = Router::new()
                      .route("/api/get_random_meme", get(get_random_meme))
                      .route("/api/add_meme", post(add_meme)) // Get the new memes that have been submitted in the frontend
                      .fallback_service(serve_dir);

    // Network adress: IP-adress + port
    // 0.0.0.0 to be reachable from outside
    let addr = SocketAddr::from(([0,0,0,0],PORT)); 

    // Listener enables servers to connect to browser, claims a port (gets it from SocketAddr)
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener,app).await.unwrap()
}
