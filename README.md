# propaganda

Articles on the internet are published but changed afterwards.
This can be surprising.
Let's show where this happens.

## tech outline

* either with tokio
    * use warp for HTTP server
    * use reqwest for HTTP client
* or with async-std
    * use tide for HTTP server
    * use surf for HTTP client

* use scraper to deal with HTML
* use sled for an prototype DB file
* use difftastic or prettydiff for diff generation
* safe snapshots of articles in the DB, generate diff on demand

## todos

* [x] fetch tageschau.de article with scraper
* [ ] store article in sled