// Include Modules -
mod discord_api;

// Crates -
use reqwest::Client;
use reqwest::header::{AUTHORIZATION, HeaderValue};
use serde_json::Value;
use tokio;
use tokio::time::{sleep, Duration};
use dotenv::dotenv;
use std::borrow::Borrow;
use std::env;
use std::sync::mpsc::channel;
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


    // Create some variables to used persistently.
    let token = Arc::new(Mutex::new(None::<String>)); // Create a shared state for the token
    let current_channel_id = Arc::new(Mutex::new(None::<SharedString>)); // Create a shared state for the token
    let mut is_logged_in = false;


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
    let current_channel_id_clone = current_channel_id.clone();
    ui.on_ServerChannelButtonClicked(move |channel_id| {
        let token: Option<String> = token_clone.lock().unwrap().clone(); // Access the token
        let channel_id = channel_id.clone();
        let ui_handle = ui_handle_clone.clone();
        
        println!("before channel id set");
        // Attempt to acquire the lock on current_channel_id
        let mut current_channel_id_guard = match current_channel_id_clone.try_lock() {
            Ok(lock) => lock,
            Err(_) => {
                eprintln!("Failed to acquire lock on current_channel_id");
                return; // Early return to avoid further processing
            }
        };
        *current_channel_id_guard = Some(channel_id.clone());
    
        println!("after channel id set");

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
                    is_logged_in = true;
                } else {
                    eprintln!("UI handle has been lost");
                }
                ()
            });
        }
    });


    

    // Creates a background task to constantly retrieve messages from the current channel
    slint::spawn_local(async move {
        println!("polling thread spawned");
        loop {
            println!("one loop is running");
            sleep(Duration::from_secs(10)).await;
            let token_clone = token.clone();
            let ui_handle_clone = ui_handle.clone();

            // Retry acquiring the lock if it fails
            let channel_id_guard = loop {
                match current_channel_id.try_lock() {
                    Ok(lock) => break lock,
                    Err(_) => {
                        eprintln!("Failed to acquire lock on channel_id, retrying...");
                        sleep(Duration::from_secs(1)).await; // Wait before retrying
                    }
                }
            };

            println!("passed channel id");

            // Extract the channel_id
            let channel_id = match &*channel_id_guard {
                Some(id) => id.clone(),
                None => {
                    eprintln!("Channel ID is not available");
                    continue; // Skip this iteration and retry
                }
            };

    
            let token_poll = match token_clone.try_lock() {
                Ok(lock) => lock.clone(),
                Err(_) => {
                    eprintln!("Failed to acquire lock on token");
                    continue; // Skip this iteration and try again
                }
            };

            println!("passed token");

            // Updates messages (full reload for now).
            match discord_api::get_messages(&token_poll.as_deref().unwrap(), &channel_id).await {
                Ok(channel_messages) => {
                    println!("Get Messages Response:\n{}", channel_messages);
                    let channel_messages_text: Vec<SharedString> = channel_messages
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|messages| SharedString::from(messages["content"].as_str().unwrap_or("")))
                        .collect();
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_channel_messages(
                            slint::ModelRc::new(slint::VecModel::from(channel_messages_text))
                        );
                    } else {
                        eprintln!("Failed to upgrade UI handle to a strong reference");
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }

            
            println!("going to sleep");
        }
    });
    
    
    // Run the Slint UI
    ui.run();


    Ok(())
}