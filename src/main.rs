extern crate reqwest;
extern crate tokio;
use std::fmt::format;
use std::io::Write;
use std::slice::Chunks;
use reqwest::header::AUTHORIZATION;
use serde_json::Value;
use std::path::Path;
use std::fs::{File, read};

#[tokio::main] 
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("hello");
    let path = Path::new("discordscraper.json");
    let token: String;
    let serverid: u64;
    let channelid: u64;
    let mut amount: i32;
    if !(path.exists()) {
        println!("couldnt find discordscraper.json");
        let mut token1 = String::new();
        print!("your token: ");
        std::io::stdout().flush().expect("failed to flush");
        std::io::stdin().read_line(&mut token1).expect("failed to read line");
        token = token1.trim().to_string();
        let mut serverid1 = String::new();
        print!("serverid: ");
        std::io::stdout().flush().expect("failed to flush");
        std::io::stdin().read_line(&mut serverid1).expect("couldnt read line");
        serverid = serverid1.trim().parse().expect("couldnt parse serverid");
        let mut channelid1 = String::new();
        print!("channelid: ");
        std::io::stdout().flush().expect("failed to flush");
        std::io::stdin().read_line(&mut channelid1).expect("couldnt read line");
        channelid = channelid1.trim().parse().expect("couldnt trim channelid");
        let mut amount1 = String::new();
        print!("amount: ");
        std::io::stdout().flush().expect("failed to flush");
        std::io::stdin().read_line(&mut amount1).expect("couldnt read line");
        amount = channelid1.trim().parse().expect("couldnt trim amount");
        } 
    else {
        let file = File::open("discordscraper.json")?;
        let filejson: Value = serde_json::from_reader(file)?;
        token = filejson["token"].to_string().replace("\"", "");
        serverid = filejson["serverid"].to_string().parse()?;
        channelid = filejson["channelid"].to_string().parse()?;
        amount = filejson["amount"].to_string().parse()?;
    }
    let mut offset: usize = 0;
    let responsejson = getresponse(&token, serverid, channelid, Some(offset)).await?;

    let msglength = downloader(responsejson, Some(amount)).await?;
    
    amount -= msglength as i32;
    if amount <= 0 {
        let mut input = String::new();
        print!("press enter to exit");
        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut input)?;
        return Ok(());
    }
    let chunks = amount / msglength as i32;
    let remainder = amount % msglength as i32;
    for _ in 0..chunks {
        offset += msglength;
        let responsejson = getresponse(&token, serverid, channelid, Some(offset)).await?;
        let msglen = downloader(responsejson, Some(amount)).await?;
        amount -= msglen as i32;
        if amount <= 0 {
            let mut input = String::new();
            print!("press enter to exit");
            std::io::stdout().flush()?;
            std::io::stdin().read_line(&mut input)?;
            return Ok(());
        }
    }
    offset += remainder as usize;
    let responsejson = getresponse(&token, serverid, channelid, Some(offset)).await?;
    let _ = downloader(responsejson, Some(amount)).await?;
    let mut input = String::new();
    print!("press enter to exit");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    Ok(())
}

async fn downloader(responsejson: Value, amount: Option<i32>) -> Result<usize, Box<dyn std::error::Error>> {
    let mut messagelen: usize = 0;
    if let Value::Object(obj) = responsejson {
        if let Some(messages) = obj.get("messages") {
            if let Value::Array(messagearray) = messages {
                println!("amount of messaegs in response: {}", messagearray.len());
                messagelen = messagearray.len();
                let mut howmany = amount.unwrap();
                for i in messagearray {
                    if howmany == 0 {
                        return Ok(messagelen);
                    }
                    howmany -= 1;
                    if let Value::Array(msg) = i {
                        for j in msg {
                            if let Value::Object(msgobj) = j {
                                if let Some(attachmentstr) = msgobj.get("attachments") {
                                    if let Value::Array(attachmentarray) = attachmentstr {
                                        if attachmentarray.len() > 0 {
                                            downloadvids(attachmentstr).await?;
                                        }
                                    }

                                    
                                }
                                if let Some(embedstr) = msgobj.get("embeds") {
                                    if let Value::Array(embedarray) = embedstr {
                                        if embedarray.len() > 0 {
                                            downloadvids(embedstr).await?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(messagelen)
}

async fn getresponse(token: &str, serverid: u64, channelid: u64, offset: Option<usize>) -> Result<Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("https://canary.discord.com/api/v9/guilds/{}/messages/search?channel_id={}&has=video&include_nsfw=true&offset={}", serverid, channelid, offset.unwrap());
    let request = client.get(&url).header(AUTHORIZATION, token);
    let response = request.send().await?;
    let responsejson: Value = serde_json::from_str(&response.text().await?)?;
    Ok(responsejson)
}
async fn downloadvids(videoarrays: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Value::Array(vidarray) = videoarrays {
        for element in vidarray {
            if let Some(contenttype) = element.get("content_type") {
                let content = contenttype.as_str().unwrap_or("idk");
                if !(content.contains("video")) {
                    continue;
                }
            }
            let mut filename: String = "unknownfilename".to_string();
            if let Some(Value::String(filenam)) = element.get("filename") {
                if !filenam.is_empty() {
                    filename = filenam.to_string();
                }
            } else {
                if let Some(videoobj) = element.get("video") {
                    if let Some(_) = videoobj.get("proxy_url") {
                        let url = videoobj["proxy_url"].as_str().unwrap().replace("\"", "");
                        let lastpart: Vec<&str> = url.split("/").collect();
                        filename = lastpart.last().unwrap().to_string();
                    } else {
                        if let Some(url) = element.get("url") {
                            let url = url.as_str().unwrap_or("failed to unwrap string");
                            println!("found embed from outside of discord: {}", url);
                            
                        } else {
                            println!("found 3rd party embed, cant get its link tho?\n{:#?}\n{:#?}", &videoarrays, &element);
                        }
                        
                        continue;
                } 
                }
                

            }
            if let Ok(_metadata) = std::fs::metadata(&filename) {
                let namefile = &filename[0..filename.rfind(".").unwrap()];
                let extension = &filename[filename.rfind(".").unwrap()..];
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("time error")
                    .as_secs();
                filename = format!("{}-{}.{}", namefile, timestamp, extension);
            }
            let mut file = File::create(&filename).expect("error creating a file");
            let url = element.get("url").unwrap().to_string().replace("\"", "");
            let mut response = reqwest::get(url).await?;
            let mut spacer: usize = 0;
            let mut contentlength: u64 = 0;
            let mut downloaded: usize = 0;
            if let Some(content_length) = response.headers().get(reqwest::header::CONTENT_LENGTH) {
                if let Ok(length) = content_length.to_str() {
                    contentlength = length.parse().unwrap();
                }
                }
            while let Some(chunk) = response.chunk().await? {
                spacer += 1;
                downloaded += &chunk.len();
                file.write(&chunk)?;
                if spacer % 2 == 0 {
                    print!("{}/{}\r", downloaded, contentlength)
                }
            }
            println!("Downloaded {}", &filename);
                

            
        }
    }
    Ok(())
}