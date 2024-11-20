use reqwest::{multipart, Client};
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde_json::Value;
use std::error::Error;
use std::time::Duration;
use serde::Deserialize;
use std::process::{Command, Stdio};
use tokio::time::sleep;
use futures_util::{StreamExt, SinkExt}; // Add these for split and send
use tokio_tungstenite::tungstenite::protocol::Message; // Correct tungstenite import
use serde_json::json;
use serde::ser::StdError;
use tokio_tungstenite::connect_async;
use std::process::Child; // To manage the child process
use std::path::PathBuf;
use std::fs;


// FUNCTION: Sends login to Discord and returns auth token and user id
pub async fn login_request(username: String, password: String) -> Result<(String, String), Box<dyn Error>> {
    let client = Client::new();
    let body = serde_json::json!({ "login": &username, "password": &password });

    let response = client
        .post("https://discord.com/api/v9/auth/login")
        .json(&body)
        .send()
        .await?;

    if response.status().is_success() {
        let json_response: Value = response.json().await?;
        let user_id = json_response["user_id"].as_str()
            .ok_or("Missing user_id in response")?
            .to_string();
        let token = json_response["token"].as_str()
            .ok_or("Missing token in response")?
            .to_string();
        Ok((user_id, token))
    } else {
        Err(format!("Login request failed with status: {}", response.status()).into())
    }
}

// FUNCTION: Get user's DM's
pub async fn get_channels(token: String) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();
    let url = "https://discord.com/api/v9/users/@me/channels";

    let response = client
        .get(url)
        .header(AUTHORIZATION, HeaderValue::from_str(&token)?)
        .send()
        .await?;

    if response.status().is_success() {
        let response_json = response.json().await?;
        Ok(response_json)
    } else {
        Err(format!("Get user's DM channels request failed with status: {}", response.status()).into())
    }
}

// FUNCTION: Gets user's servers (guilds)
pub async fn get_guilds(token: String) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();

    let response = client
        .get("https://discord.com/api/v9/users/@me/guilds")
        .header(AUTHORIZATION, HeaderValue::from_str(&token)?)
        .send()
        .await?;

    if response.status().is_success() {
        let response_json = response.json().await?;
        Ok(response_json)
    } else {
        Err(format!("Get servers request failed with status: {}", response.status()).into())
    }
}

// FUNCTION: Get server channels
pub async fn get_guild_channels(token: String, guild_id: String) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();
    let url = format!("https://discord.com/api/v9/guilds/{}/channels", guild_id);

    let response = client
        .get(&url)
        .header(AUTHORIZATION, HeaderValue::from_str(&token)?)
        .send()
        .await?;

    if response.status().is_success() {
        let response_json = response.json().await?;
        Ok(response_json)
    } else {
        Err(format!("Get guild channels request failed with status: {}", response.status()).into())
    }
}

// FUNCTION: Sends message to a server channel
pub async fn send_message(token: String, channel_id: String, message: String) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();
    let url = format!("https://discord.com/api/v9/channels/{}/messages", channel_id);
    let body = serde_json::json!({ "content": message });

    let response = client
        .post(&url)
        .header(AUTHORIZATION, HeaderValue::from_str(&token)?)
        .json(&body)
        .send()
        .await?;

    if response.status().is_success() {
        let response_json = response.json().await?;
        Ok(response_json)
    } else {
        Err(format!("Send message request failed with status: {}", response.status()).into())
    }
}

// FUNCTION: Sends message to a server channel
pub async fn send_message_attachment(token: String, channel_id: String, message: String, attachment: Vec<u8>, attachment_name: String) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();
    let url = format!("https://discord.com/api/v9/channels/{}/messages", channel_id);
    let body = serde_json::json!({ "content": message });

    // Create a multipart form with the file content and the message content
    let form = multipart::Form::new()
        .text("content", message) // Add the message content as a text part
        .part(
            "files[0]", // The name of the part that Discord expects for file attachments
            multipart::Part::bytes(attachment)
                .file_name(attachment_name), // Add the file as a multipart part with a file name
        );

    let response = client
        .post(&url)
        .header(AUTHORIZATION, HeaderValue::from_str(&token)?)
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        let response_json = response.json().await?;
        Ok(response_json)
    } else {
        Err(format!("Send message request failed with status: {}", response.status()).into())
    }
}

// FUNCTION: Sends a reaction to a message in a server channel
pub async fn send_reaction(token: String, channel_id: String, message_id: String, emoji: String) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let emoji_clean = strip_quotes(&emoji); // Remove quotes if present
    let message_id_clean = strip_quotes(&message_id);

    let url = format!(
        "https://discord.com/api/v9/channels/{}/messages/{}/reactions/{}/@me",
        channel_id, message_id_clean, emoji_clean
    );
    println!("send_reaction received emoji: {}", emoji_clean);
    println!("url is: {}", url);

    let response = client
        .put(&url)
        .header(AUTHORIZATION, HeaderValue::from_str(&token)?)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Send reaction request failed with status: {}", response.status()).into())
    }
}


// FUNCTION: Get messages from a channel
pub async fn get_messages(token: String, channel_id: String) -> Result<Value, Box<dyn Error>> {
    let client = Client::builder()
    .timeout(Duration::from_secs(2)) // Set a 2-second timeout
    .build()?;
    let url = format!("https://discord.com/api/v9/channels/{}/messages", channel_id);
    
    println!("in get messages");

    let response = client
        .get(&url)
        .header(AUTHORIZATION, HeaderValue::from_str(&token)?)
        .send()
        .await?;

    if response.status().is_success() {
        println!("response success");
        let response_json = response.json().await?;
        Ok(response_json)
    } else {
        Err(format!("Get messages request failed with status: {}", response.status()).into())
    }
}

fn strip_quotes(s: &str) -> &str {
    s.trim_matches('"')
}

#[derive(Deserialize)]
struct Tab {
    id: String,
    webSocketDebuggerUrl: String,
}

pub async fn launch_chrome_and_monitor_auth() -> Result<Option<String>, Box<dyn StdError>> {
    // Step 1: Specify the path to your existing Chrome profile
    let start_url = "https://discord.com/login";

    // Step 2: Open Chrome with remote debugging enabled and use the existing profile
    let chrome_path = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
    //let user_data_dir = r"C:\TempProfile"; // Path to the newly copied Chrome profile
    let user_data_dir = std::env::current_dir()
        .map(|path| path.join("resources").join("TempProfile"))
        .unwrap_or_else(|_| PathBuf::from("resources/TempProfile"));

    // Convert to a string for passing to the command
    let user_data_dir_str = user_data_dir.to_str().unwrap_or_else(|| {
        panic!("Failed to convert user_data_dir to a string");
    });

    // Launch Chrome with remote debugging enabled
    let mut command = Command::new(chrome_path)
    .arg(format!("--user-data-dir={}", user_data_dir_str))
    .arg("--remote-debugging-port=9222")
    .arg("--new-instance") // Ensure Chrome opens a new window
    .arg("--no-first-run")
    .arg("--no-default-browser-check") // Skip default browser prompt
    .arg("--disable-extensions") // Optional: Disable extensions
    .arg("--incognito")
    .arg(start_url)
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn();

    let mut chrome_process: Option<Child> = match command {
        Ok(child) => Some(child),
        Err(e) => {
            eprintln!("Failed to start Chrome: {}", e);
            return Ok(None);
        }
    };

    // Wait for Chrome to start
    println!("Waiting for Chrome to start...");
    sleep(Duration::from_secs(2)).await; // Wait 2 seconds for Chrome to fully start

    // Query the debugging endpoint to get the WebSocket debugger URL
    let url = "http://localhost:9222/json";
    let client = Client::new();
    let response = client.get(url).send().await?;

    if response.status().is_success() {
        let tabs: Vec<Tab> = response.json().await?;
        if let Some(tab) = tabs.first() {
            let websocket_url = &tab.webSocketDebuggerUrl;
            println!("WebSocket Debugger URL: {}", websocket_url);

            // Now, monitor the network traffic and capture the authorization header
            let (ws_stream, _) = connect_async(websocket_url).await?;
            let (mut write, mut read) = ws_stream.split();

            // Enable Network domain in CDP
            let enable_network = json!({
                "id": 1,
                "method": "Network.enable",
            });
            write.send(Message::Text(enable_network.to_string())).await?;
            println!("Sent Network.enable command");

            // Monitor WebSocket messages for the authorization header
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let parsed: serde_json::Value = match serde_json::from_str(&text) {
                            Ok(parsed) => parsed,
                            Err(e) => {
                                eprintln!("Failed to parse WebSocket message: {}", e);
                                continue;
                            }
                        };
                        
                        // Log all received messages for debugging
                        println!("Received message: {}", serde_json::to_string_pretty(&parsed).unwrap());
        
                        // Check for 'Network.requestWillBeSent' or 'Network.responseReceived' methods
                        if let Some(method) = parsed["method"].as_str() {
                            if method == "Network.responseReceived" || method == "Network.requestWillBeSent" {
                                if let Some(params) = parsed["params"].as_object() {
                                    // Handle response data safely by checking the key existence
                                    if let Some(request) = params.get("request") {
                                        if let Some(headers) = request.get("headers").and_then(|h| h.as_object()) {
                                            if let Some(auth_header) = headers.get("Authorization") {
                                                println!("Authorization Header Found: {}", auth_header);
                                                let token = auth_header.as_str().unwrap_or_default().to_string();
                                                // Close the Chrome window and end the program
                                                if let Some(mut process) = chrome_process.take() {
                                                    process.kill().expect("Failed to kill Chrome process");
                                                    println!("Chrome window closed.");
                                                }
                                                return Ok(Some(token));  // Return the captured token
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => eprintln!("Error reading WebSocket message: {}", e),
                    _ => {}
                }
            }
        }
    }

    Err("Failed to retrieve the WebSocket URL or capture authorization header.".into())
}