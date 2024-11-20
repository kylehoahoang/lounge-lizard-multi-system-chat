use dioxus::prelude::*;
use crate::{MONGO_COLLECTION, MONGO_DATABASE};
use bson::to_bson;

use crate::api::discord::discord_api;
use futures::executor::block_on;
use serde_json::Value;
use crate::api::mongo_format::mongo_structs::*;
use mongodb::{sync::Client, bson::doc};

use dioxus_logger::tracing::{info, error, warn};

use std::sync::Arc;
use tokio::sync::Mutex;

#[component]
pub fn DiscordLogin(

    show_discord_login_pane: Signal<bool>,
    show_discord_server_pane: Signal<bool>,
    discord_token: Signal<String>,
    discord_guilds: Signal<Value>, 
    current_platform: Signal<String>, 
   
)
-> Element {
    // ! User Mutex Lock to access the user data
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    let client_lock = use_context::<Signal<Arc<Mutex<Option<Client>>>>>();
    // ! ========================= ! //

    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());

    let mut logged_in = use_signal(|| false);

    let mut login_error = use_signal(|| None::<String>);

    let handle_login = move |_| {
        let username = username.clone();
        let password = password.clone();

        block_on(async move {
            match discord_api::login_request(username.to_string(), password.to_string()).await {
                Ok((_user_id, auth_discord_token)) => {
                    discord_token.set(auth_discord_token); // Call the success handler
                    show_discord_login_pane.set(false);
                    show_discord_server_pane.set(true);
                    info!("Login successful");
                }
                Err(e) => {
                    login_error.set(Some(e.to_string()));
                    info!("Login failed: {}", e);
                }
            }
        });

        if !login_error().is_none() {return;}

        let mongo_lock_copies = client_lock().clone();
        let user_lock_copies = user_lock().clone();

        let mongo_client = block_on(
            async {
                mongo_lock_copies.lock().await
            }
        );

        let mut user = block_on(
            async {
                user_lock_copies.lock().await
            }
        );
         // Clone the client if it exists (since we can't return a reference directly)
        if let Some(client) = mongo_client.as_ref() {
            // Convert the function into async and spawn it on the current runtime
            let client_clone = client.clone();  // Clone the client to avoid ownership issues

            // TODO Add all tokens to user profile here
            user.discord = Discord{
                token: discord_token.to_string()
            };
            
            // Todo ====================================//

            let user_clone = user.clone();
            
            // Use `tokio::spawn` to run the async block
            block_on(async move {
                let db = client_clone.database(MONGO_DATABASE);
                let user_collection = db.collection::<User>(MONGO_COLLECTION);
                
                match to_bson(&user_clone.discord) {
                    Ok(discord_bson) => {
                        match user_collection
                            .find_one_and_update(
                                doc! { 
                                    "$or": [{"username": &user_clone.username}, 
                                            {"email": &user_clone.email}] 
                                },
                                doc! {
                                    "$set": { "discord": discord_bson }
                                }
                            )
                            .await 
                        {
                            Ok(Some(_)) => {
                                // Document found and updated
                                info!("Document updated successfully");
                                logged_in.set(true);
                                current_platform.set("Discord".to_string());
                            }
                            Ok(None) => {
                                // No document matched the filter
                                warn!("Document not found");
                            }
                            Err(e) => {
                                error!("Something went wrong: {:#?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to convert Slack to BSON: {:#?}", e);
                    }
                }
                
            });

        } else {
            warn!("MongoDB client not found in global state.");
        }
    };

    let handle_browser_login = move |_| {
        block_on(async move {
            // Launch Chrome and capture the WebSocket debugger URL
            let existing_profile_path = "resources/chrome_empty_profile";
            let chrome_path = "C:/Program Files/Google/Chrome/Application/chrome.exe";
            let start_url = "https://discord.com/login";

            let captured_token = match discord_api::launch_chrome_and_monitor_auth().await {
                Ok(Some(token)) => {
                    println!("Captured Token: {}", token);
                    Some(token)
                }
                Ok(None) => {
                    warn!("No token found during monitoring");
                    None
                }
                Err(e) => {
                    error!("Failed to launch Chrome and monitor WebSocket for token: {}", e);
                    None
                }
            };
            

            let auth_discord_token = if let Some(token) = captured_token {
                token
            } else {
                login_error.set(Some("login failed".to_string()));
                return;
            };

            discord_token.set(auth_discord_token.clone());
            show_discord_login_pane.set(false);
            show_discord_server_pane.set(true);
            info!("Login successful");
        });

        if !login_error().is_none() {
            return;
        }

        let mongo_lock_copies = client_lock().clone();
        let user_lock_copies = user_lock().clone();

        let mongo_client = block_on(
            async {
                mongo_lock_copies.lock().await
            }
        );

        let mut user = block_on(
            async {
                user_lock_copies.lock().await
            }
        );
         // Clone the client if it exists (since we can't return a reference directly)
        if let Some(client) = mongo_client.as_ref() {
            // Convert the function into async and spawn it on the current runtime
            let client_clone = client.clone();  // Clone the client to avoid ownership issues

            // TODO Add all tokens to user profile here
            user.discord = Discord{
                token: discord_token.to_string()
            };
            
            // Todo ====================================//

            let user_clone = user.clone();
            
            // Use `tokio::spawn` to run the async block
            block_on(async move {
                let db = client_clone.database(MONGO_DATABASE);
                let user_collection = db.collection::<User>(MONGO_COLLECTION);
                
                match to_bson(&user_clone.discord) {
                    Ok(discord_bson) => {
                        match user_collection
                            .find_one_and_update(
                                doc! { 
                                    "$or": [{"username": &user_clone.username}, 
                                            {"email": &user_clone.email}] 
                                },
                                doc! {
                                    "$set": { "discord": discord_bson }
                                }
                            )
                            .await 
                        {
                            Ok(Some(_)) => {
                                // Document found and updated
                                info!("Document updated successfully");
                                logged_in.set(true);
                                current_platform.set("Discord".to_string());
                            }
                            Ok(None) => {
                                // No document matched the filter
                                warn!("Document not found");
                            }
                            Err(e) => {
                                error!("Something went wrong: {:#?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to convert Slack to BSON: {:#?}", e);
                    }
                }
                
            });

        } else {
            warn!("MongoDB client not found in global state.");
        }
    };

    rsx! {
        div {
            class: format_args!("discord-login {}", if show_discord_login_pane() { "visible" } else { "" }),
            img {
                src: "assets/discord_logo.png",
                alt: "Discord Logo",
                width: "50px",
                height: "50px",
            }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| {show_discord_login_pane.set(false) },
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    view_box: "0 0 24 24",
                    width: "30", // Adjust size as needed
                    height: "30", // Adjust size as needed
                    path {
                        d: "M18 6 L6 18 M6 6 L18 18", // This path describes a close icon (X)
                        fill: "none",
                        stroke: "#f5f5f5", // Change stroke color as needed
                        stroke_width: "2" // Adjust stroke width
                    }
                }
            }
            input {
                class: "login-input",
                value: "{username}",
                placeholder: "Username/Email",
                oninput: move |event| username.set(event.value())
            }
            input {
                class: "login-input",
                r#type: "password",
                value: "{password}",
                placeholder: "Password",
                oninput: move |event| password.set(event.value())
            }
            button { 
                class: "login-button",
                onclick: handle_login, "Login" 
            }
            button { 
                class: "login-button",
                onclick: handle_browser_login, "Browser Login (Requires Chrome)" 
            }

            // TODO: provide custom error warnings
            if let Some(error) = login_error() {
                p { 
                    style: "color: white; font-family: Arial, sans-serif; font-weight: bold; text-align: center;",
                    "Login failed: {error}" 
                }
            }
        }
    }
}

