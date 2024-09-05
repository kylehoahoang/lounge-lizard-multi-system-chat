use reqwest::Client;
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde_json::Value;
use tokio;
use dotenv::dotenv;
use std::env;
use slint::{ComponentHandle, SharedString};
use std::sync::{Arc, Mutex};
use std::sync::MutexGuard;
// slint::include_modules!();
slint::slint!{ import { MainWindow } from "src/ui/main.slint";}

// FUNCTION: Sends login to Discord and returns auth token and user id
async fn login_request(username: String, password: String) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Create a client instance
    let client = Client::new();
    println!("Hello from login request");

    let body = serde_json::json!({
        "login": &username,
        "password": &password
    });

    let response = client
        .post("https://discord.com/api/v9/auth/login")
        .json(&body) // Send the JSON body
        .send()
        .await?;

    if response.status().is_success() {
        // Deserialize the response to a `serde_json::Value`
        let json_response: Value = response.json().await?;
        
        // Extract user_id and token from the JSON response
        let user_id = json_response["user_id"].as_str()
            .ok_or("Missing user_id in response")?
            .to_string();
        let token = json_response["token"].as_str()
            .ok_or("Missing token in response")?
            .to_string();
        
        // Return user_id and token as a tuple
        Ok((user_id, token))
    } else {
        Err(format!("Login request failed with status: {}", response.status()).into())
    }
}

// FUNCTION: Get's user's DM's
async fn get_channels(token: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();

    let url = format!("https://discord.com/api/v9/users/@me/channels");

    let response = client
        .get(&url)
        .header(AUTHORIZATION, HeaderValue::from_str(token)?)
        .send()
        .await?;

    if response.status().is_success() {
        let response_json = response.json().await?;
        // println!("Response: {}", response_json);
        Ok(response_json)
    } else {
        Err(format!("Get user's DM channels request failed with status: {}", response.status()).into())
    }
}


// FUNCTION: Gets user's servers (guilds)
async fn get_guilds(token: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();

    let response = client
        .get("https://discord.com/api/v9/users/@me/guilds")
        .header(AUTHORIZATION, HeaderValue::from_str(token)?)
        .send()
        .await?;

    if response.status().is_success() {
        let response_json = response.json().await?;
        // println!("Response: {}", response_json);
        Ok(response_json)
    } else {
        Err(format!("Get servers request failed with status: {}", response.status()).into())
    }
}

// FUNCTION: Get's server's channels
async fn get_guild_channels(token: &str, guild_id: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();

    let url = format!("https://discord.com/api/v9/guilds/{}/channels", guild_id);

    let response = client
        .get(&url)
        .header(AUTHORIZATION, HeaderValue::from_str(token)?)
        .send()
        .await?;

    if response.status().is_success() {
        let response_json = response.json().await?;
        // println!("Response: {}", response_json);
        Ok(response_json)
    } else {
        Err(format!("Get guild channels request failed with status: {}", response.status()).into())
    }
}

// FUNCTION: Sends message to a server channel
async fn send_message(token: &str, channel_id: &str, message: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();

    let url = format!("https://discord.com/api/v9/channels/{}/messages", channel_id);

    let body = serde_json::json!({
        "content": message
    });

    let response = client
        .post(&url)
        .header(AUTHORIZATION, HeaderValue::from_str(token)?)
        .json(&body)
        .send()
        .await?;

    if response.status().is_success() {
        let response_json = response.json().await?;
        // println!("Response: {}", response_json);
        Ok(response_json)
    } else {
        Err(format!("Send message request failed with status: {}", response.status()).into())
    }
}

// FUNCTION: Sends message to a server channel
async fn get_messages(token: &str, channel_id: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();

    let url = format!("https://discord.com/api/v9/channels/{}/messages", channel_id);

    let response = client
        .get(&url)
        .header(AUTHORIZATION, HeaderValue::from_str(token)?)
        .send()
        .await?;

    if response.status().is_success() {
        let response_json = response.json().await?;
        // println!("Response: {}", response_json);
        Ok(response_json)
    } else {
        Err(format!("Send message request failed with status: {}", response.status()).into())
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env variables
    dotenv().ok();

    // Create the Slint UI
    let ui = MainWindow::new().unwrap();

    let token = Arc::new(Mutex::new(None)); // Create a shared state for the token
    let ui_handle = Arc::new(ui.as_weak());

    // Set up event handler for button clicks
    let token_clone = token.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_ServerButtonClicked(move |server_id| {
        let token = token_clone.lock().unwrap().clone(); // Access the token
        let server_id = server_id.clone();
        let ui_handle = ui_handle_clone.clone();

        slint::spawn_local(async move {
            if let Some(ui) = ui_handle.upgrade() {
                match get_guild_channels(&token.as_deref().unwrap(), &server_id).await {
                    Ok(server_channels) => {
                        println!("Server Channels Response:\n{}", server_channels);
                        let server_channel_names: Vec<SharedString> = server_channels
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .map(|channel| SharedString::from(channel["name"].as_str().unwrap_or("")))
                            .collect();
                        let server_channel_ids: Vec<SharedString> = server_channels
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .map(|channel| SharedString::from(channel["id"].as_str().unwrap_or("")))
                            .collect();
                        ui.set_server_channels_names(
                            slint::ModelRc::new(slint::VecModel::from(server_channel_names))
                        );
                        ui.set_server_channels_ids(
                            slint::ModelRc::new(slint::VecModel::from(server_channel_ids))
                        );
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            } else {
                eprintln!("UI handle has been lost");
            }
        });
    });

    // Set up event handler for channel button clicks
    let token_clone = token.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_ServerChannelButtonClicked(move |channel_id| {
        let token = token_clone.lock().unwrap().clone(); // Access the token
        let channel_id = channel_id.clone();
        let ui_handle = ui_handle_clone.clone();
        println!("channel id selected:\n{}", channel_id);

        slint::spawn_local(async move {
            if let Some(ui) = ui_handle.upgrade() {
                match get_messages(&token.as_deref().unwrap(), &channel_id).await {
                    Ok(channel_messages) => {
                        println!("Get Messages Response:\n{}", channel_messages);
                        let channel_messages_text: Vec<SharedString> = channel_messages
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .map(|messages| SharedString::from(messages["content"].as_str().unwrap_or("")))
                            .collect();
                        ui.set_channel_messages(
                            slint::ModelRc::new(slint::VecModel::from(channel_messages_text))
                        );
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            } else {
                eprintln!("UI handle has been lost");
            }
        });
    });



    let token_clone = token.clone();
    // Set up event handler for channel button clicks
    let ui_handle_clone = ui_handle.clone();
    ui.on_process_login(move |username, password| {
        // println!("username from login is: {}", username);
        // println!("Password from login is: {}", password);
        let ui_handle = ui_handle_clone.clone();
        let token = token_clone.clone();
        if let Some(ui) = ui_handle.upgrade() {
            slint::spawn_local(async move {
                if let Some(ui) = ui_handle.upgrade() {
                    // Call login function to get auth token
                    let (user_id, token_value) = match login_request(username.to_string(), password.to_string()).await {
                        Ok((user_id, token)) => {
                            // println!("User ID: {}", user_id);
                            // println!("Token: {}", token);
                            (user_id, token)
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            return; // Ensure return type is consistent
                        }
                    };

                    // Store the token in the shared state
                    let mut token_guard: MutexGuard<Option<String>> = token.lock().unwrap();
                    *token_guard = Some(token_value.clone());

                    // Get user server list using auth token
                    let guilds = match get_guilds(&token_value).await {
                        Ok(guilds) => {
                            println!("Guilds Response:\n{}", guilds);
                            print!("\n");
                            (guilds)
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            return; // Ensure return type is consistent
                        }
                    };

                    // Get user server list using auth token
                    match get_channels(&token_value).await {
                        Ok(dm_channels) => {
                            println!("DM Channels Response:\n{}", dm_channels);
                            print!("\n");
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }

                    // Extract server names from the JSON response and convert them to SharedString
                    let server_names: Vec<SharedString> = guilds.as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|guild| SharedString::from(guild["name"].as_str().unwrap_or("")))
                    .collect();

                    // Extract server names from the JSON response and convert them to SharedString
                    let server_ids: Vec<SharedString> = guilds.as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|guild| SharedString::from(guild["id"].as_str().unwrap_or("")))
                        .collect();

                    // Set the server names in the UI
                    ui.set_server_names(slint::ModelRc::new(slint::VecModel::from(server_names)));
                    ui.set_server_ids(slint::ModelRc::new(slint::VecModel::from(server_ids)));
                    ui.invoke_toggle_show_login();
                } else {
                    eprintln!("UI handle has been lost");
                }
                ()
            });
        }
    });

    // Run the Slint event loop
    ui.run();

    Ok(())
}