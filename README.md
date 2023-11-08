# Simple command line discord video scraper
## How to use
### 1. Create a file named discordscraper.json, put it in the same directory as the exe and put in the details:
```json
{"serverid": "serverid", 
"channelid": "channelid", 
"token": "account token",
"amount": "amount of messages"}
```
### Or just run the exe and it will ask you for the details

## How to compile the code (create an exe)
### 1. go to [https://rustup.rs](https://rustup.rs) 
### or 
### [https://rust-lang.github.io/rustup/installation/other.html](https://rust-lang.github.io/rustup/installation/other.html)
### 2. cd to the discordvidscraper directory, run
```bash
cargo build --release
```
### 3. you will find the exe in the /release directory, you can put that exe anywhere, but make sure the discordscraper.json file is in the same directory as the executable