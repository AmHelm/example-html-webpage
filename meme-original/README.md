# An example webpage for HTML practice

This repository contains code developed as HTML practice during my internship. This is how I made my first website.

## Installing a web server

The website is hosted through a remote server and so nginx was installed and used to make the website available from outside, accessible through the ip-adress.

Install nginx unless already installed.
```bash
sudo apt update
sudo apt install -y nginx
```

Check if nginx is working by accessing the website. There should be a "Welcome to nginx!" page if the server is serving web trafic. Replace IP-NUMBER with the server's ip-adress.
http://IP-NUMBER

## Building the code

The aim was to get familiar with HTML, and so I tried out different simple attributes by scripting in VSCode and checking out the results in my web browser.

To make sure that nginx reads the HTML file correctly it was named index.html.

## Updating script and website

The work-in-progress script is stored on a local host, so when launching a new version of the website, the new script has to be pushed to the remote server.

If not already in place on the remote server, enter the server and add a directory to store the file. Replace SERVERPATH with the path to where the HTML file will be on the server. Then exit the server.
```bash
ssh build
mkdir -p /tmp/SERVERPATH
exit
```

On the local device, copy the HTML file to the server. Replace HTMLFILEPATH with the path to the original script and NAME with the server name.
```bash
scp -r ~/HTMLFILEPATH/* NAME@IP-NUMBER:/tmp/SERVERPATH/
```

Enter the remote server again.
```bash
ssh build
```

Then, on the remote server, move the file into place. 
```bash
sudo rm -rf /var/www/html/*
sudo cp -r /tmp/SERVERPATH/* /var/www/html/
sudo chown -R www-data:www-data /var/www/html
```

Check the website ip in bowser again to make sure the changes went through.
http://IP-NUMBER

With this method the local HTML script can be changed and things can be tested out on the local host, while the online webpage only shows a final product.