// Include Modules -
mod discord_api;

// Crates -
use reqwest::Client;
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde_json::Value;
use tokio;
use dotenv::dotenv;
use std::env;
use slint::{ComponentHandle, SharedString};
use std::sync::{Arc, Mutex};
use std::sync::MutexGuard;

// Slint UI Import -
slint::slint!{ import { MainWindow } from "src/ui/main.slint";}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load any .env variables.
    dotenv().ok();


    // Create the Slint UI and a UI handle to manipulate in functions.
    let ui = MainWindow::new().unwrap();
    let ui_handle = Arc::new(ui.as_weak());


    // Create variable for token.
    let token = Arc::new(Mutex::new(None)); // Create a shared state for the token


    // Set up event handler for server button clicks.
    let token_clone = token.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_ServerButtonClicked(move |server_id| {
        let token = token_clone.lock().unwrap().clone(); // Access the token
        let server_id = server_id.clone();
        let ui_handle = ui_handle_clone.clone();

        slint::spawn_local(async move {
            if let Some(ui) = ui_handle.upgrade() {
                match discord_api::get_guild_channels(&token.as_deref().unwrap(), &server_id).await {
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

    
    // Set up event handler for channel button clicks.
    let token_clone = token.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_ServerChannelButtonClicked(move |channel_id| {
        let token = token_clone.lock().unwrap().clone(); // Access the token
        let channel_id = channel_id.clone();
        let ui_handle = ui_handle_clone.clone();

        slint::spawn_local(async move {
            if let Some(ui) = ui_handle.upgrade() {
                match discord_api::get_messages(&token.as_deref().unwrap(), &channel_id).await {
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


    // Set up event handler for processing a message send.
    let token_clone = token.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_send_message(move |channel_id, message| {
        let token = token_clone.lock().unwrap().clone(); // Access the token
        let channel_id = channel_id.clone();
        let message = message.clone();
        let ui_handle = ui_handle_clone.clone();

        slint::spawn_local(async move {
            if let Some(ui) = ui_handle.upgrade() {
                match discord_api::send_message(&token.as_deref().unwrap(), &channel_id, &message).await {
                    Ok(channel_messages) => {
                        println!("Send Messages Response:\n{}", channel_messages);
                        // let channel_messages_text: Vec<SharedString> = channel_messages
                        //     .as_array()
                        //     .unwrap_or(&vec![])
                        //     .iter()
                        //     .map(|messages| SharedString::from(messages["content"].as_str().unwrap_or("")))
                        //     .collect();
                        // ui.set_channel_messages(
                        //     slint::ModelRc::new(slint::VecModel::from(channel_messages_text))
                        // );
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }

                // Updates messages (full reload for now).
                match discord_api::get_messages(&token.as_deref().unwrap(), &channel_id).await {
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


    // Set up event handler for login event.
    let token_clone = token.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_process_login(move |username, password| {
        let ui_handle = ui_handle_clone.clone();
        let token = token_clone.clone();

        if let Some(ui) = ui_handle.upgrade() {
            slint::spawn_local(async move {
                if let Some(ui) = ui_handle.upgrade() {
                    // Call login function to get auth token
                    let (user_id, token_value) = match discord_api::login_request(username.to_string(), password.to_string()).await {
                        Ok((user_id, token)) => {
                            // println!("User ID: {}", user_id);
                            // println!("Token: {}", token);
                            (user_id, token)
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ui.invoke_unsuccessful_login();
                            return; // Ensure return type is consistent
                        }
                    };

                    // Store the token in the shared state
                    let mut token_guard: MutexGuard<Option<String>> = token.lock().unwrap();
                    *token_guard = Some(token_value.clone());

                    // Get user server list using auth token
                    let guilds = match discord_api::get_guilds(&token_value).await {
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
                    match discord_api::get_channels(&token_value).await {
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
                    ui.invoke_successful_login();
                } else {
                    eprintln!("UI handle has been lost");
                }
                ()
            });
        }
    });


    // Run the Slint UI
    ui.run();

    Ok(())
}