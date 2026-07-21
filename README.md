# Adding a rust backend to serve the static html website

So far, the website has just been a static html website, served by nginx. Now we want the website to have and actual backend, using an axum webserver to make it accessible from the outside. 

## Setting things up

First and foremost, make sure that there is a new project directory and create a new rust project.
```bash
cargo new example-backend
```
This initiates a new directory containing a new src/main.rs file, as well as Cargo.toml and Cargo.lock files. 

Now, go into the example-backend directory and import the html file, so that it can be used by the program. For example, this time I cloned the GitHub repository containing the index.html file.
```bash
cd example-backend
gh repo clone username/repositoryname # I used AmHelm/example-html-webpage
```

## Adding dependencies

To add the dependencies to the Cargo.toml file, let cargo add them.
Inside the example-backend directory, use:
```bash
cargo add axum
cargo add tokio --features full
cargo add tower-http --features fs
```
You can also add the dependencies manually to the Cargo.toml file, under [dependencies].

Keep adding dependencies in this way as the program is expanded upon.

## Structure of the code

To build a working code we need to understand the building blocks.
Here are some good references on how to build a program like this:
https://docs.rs/axum/latest/axum/struct.Router.html
https://oneuptime.com/blog/post/2026-01-25-fast-http-router-axum-rust/view
https://github.com/tokio-rs/axum/blob/main/examples/static-file-server/src/main.rs

We need ServeDir from tower-http to find the index.html file and serve it, as ServeDir can serve an entire directory of static files. The .not_found_service() method handles a fallback if the file cannot be found. 

We use Router to tell the program where requests should go, in this case the html page.

SocketAddr give the full network adress, with an IP-adress and a port number.

The listener makes it so that the servers can connect to networks/browsers by claiming a port, in this case using the IP-adress and port number from SocketAddr.

Finally, we use axum to serve the webpage.

Run the program and check the printed url to see if the webpage works.

The previously hardcoded text strings have been moved from the frontend to the backend. For that to work the text strings are stored in the meme-texts.md file. The backend can read a file line-by-line and then wrap it into Json format thanks to the read_memes_from_file() and text_memes_to_json() functions. 
Here is a good video tutorial: https://www.youtube.com/watch?v=cJLRKj_N1dw

The frontend makes a fetch() call to the backend to get the Json formatted text strings at "/api/memes".
Reference: https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API/Using_Fetch

The Backend now randomizes which meme text is sent to the frontend when the user press the button, instead of the frontend performing the randomizing function.

Users can submit new memes/text strings. When they do so the data is stored as Json in the url + /api/add_meme. The program then gets this information and sends it off to add_meme(), where the function loads the Json data in the for of the NewMeme struct.

The NewMeme struct tells the program what the format of the memes will be. It also deserializes the data through the serde::Deserialize command before the struct definition. In the future this struct can be edited to include other forms of data.

The add_meme() function then inputs the data into the new_meme_to_file() function, where the data is added to the meme-text.md file. writeln!() adds the text in a new line, which is important since the program read the file line-by-line in read_memes_from_file().
Here is a good reference for file handling: https://www.programiz.com/rust/file-handling

## Unit testing

It is good pactice to test your functions, making sure that they actually perform their tasks correctly. Unit testing has been added for the validate_meme() function, checking if the StatusCodes are being returned in the way intended.The tests check that BAD_REQUEST is being sent for invalid submissions and there is a happy-case test checking if it works for valid submissions. To run the tests, simply enter
```bash
cargo test
```
into the terminal and make sure the tests pass.

## Putting the project on the remote server

When the code is ready, put it in the remote server, go to the server and run it. 

First, access the server and make sure there is a directory to store the files.
(Also, make sure that rust and other essential packages are installed on the remote server)
```bash
ssh servername@IP-ADRESS # Replace servername and IP-ADRESS with the remote server ids
mkdir -p ~/projects/example-backend
exit
```
Then copy the files from the local devide to the remote server.
```bash
scp -r Cargo.toml Cargo.lock src meme-original servername@IP-ADRESS:~/projects/example-backend/
```
Then go back on the server, enter the directory and run the program.
```bash
ssh servername@IP-ADRESS
cd projects/example-backend
cargo run
```
Now the website should be accessible through http://IP-ADRESS:3000!
NOTE: The website will only work while the program is active in the remote server session!