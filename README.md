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