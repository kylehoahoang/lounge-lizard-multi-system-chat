use reqwest::Client;
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
pub async fn send_message(token: &str, channel_id: &str, message: &str) -> Result<Value, Box<dyn Error>> {
    let client = Client::new();
    let url = format!("https://discord.com/api/v9/channels/{}/messages", channel_id);
    let body = serde_json::json!({ "content": message });

    let response = client
        .post(&url)
        .header(AUTHORIZATION, HeaderValue::from_str(token)?)
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

// FUNCTION: Get messages from a channel
pub async fn get_messages(token: &str, channel_id: &str) -> Result<Value, Box<dyn Error>> {
    let client = Client::builder()
    .timeout(Duration::from_secs(2)) // Set a 2-second timeout
    .build()?;
    let url = format!("https://discord.com/api/v9/channels/{}/messages", channel_id);
    
    println!("in get messages");

    let response = client
        .get(&url)
        .header(AUTHORIZATION, HeaderValue::from_str(token)?)
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