use reqwest::{multipart, Client};
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde_json::Value;
use std::error::Error;
use std::time::Duration;

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
pub async fn get_channels(token: &str) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();
    let url = "https://discord.com/api/v9/users/@me/channels";

    let response = client
        .get(url)
        .header(AUTHORIZATION, HeaderValue::from_str(token)?)
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
