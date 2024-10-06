use dioxus::prelude::*;
use bson::to_bson;
use crate::api::slack;
use crate::api::slack::server_utils::setup_server::create_slack_app;
use crate::{AppRoute};

use clipboard_rs::{Clipboard, ClipboardContext};
use crate::api::mongo_format::mongo_structs::*;
use crate::api::mongo_format::mongo_funcs::*;   
use dioxus_logger::tracing::{info, error, warn};
use futures::executor::block_on;
use mongodb::{sync::Client, bson::doc};

use std::sync::{Arc};
use tokio::sync::Mutex;

use std::fs;
use url::Url; 

use slack_morphism::prelude::*;
use crate::api::slack::ngrok_s::*;

#[component]

pub fn SlackLogin (
    show_slack_login_pane: Signal<bool>,
    current_platform: Signal<String>,
) -> Element {

    // ! User Mutex Lock to access the user data
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    let client_lock = use_context::<Signal<Arc<Mutex<Option<Client>>>>>();
    // ! ========================= ! //

    let mut logged_in = use_signal(||false);
    let mut login_error = use_signal(|| None::<String>);

    // ! Slack Temp Values ! //
    let mut oauth_url = use_signal(|| String::new());

    let handle_new_user = move |_| {

        // Collect the Slack config token from the clipboard
        // and store it in the `User` struct
        let login = block_on(async move {

            let ctx = ClipboardContext::new()
                                            .unwrap()
                                            .get_text()
                                            .unwrap_or(
                                                "".to_string()
                                            );

            // Create a new Slack app using the provided config token
            // and store the app data in the `User` struct
            create_slack_app(user_lock().clone(), ctx).await
        });

         // Handle the result of the login operation
        match login {
            Ok(_) => {
                println!("Login successful!");
                login_error.set(None);
                logged_in.set(true);
                // Continue with the rest of your program logic here
            }
            Err(e) => {
                error!("Login failed: {}", e);
                login_error.set(Some(e.to_string()));
                return;
                // Handle the error, possibly exit or retry
            }
        }

        let mongo_lock_copies = client_lock().clone();
        
        let mongo_client = block_on(
            async {
                mongo_lock_copies.lock().await
            }
        );

        let user_lock_copies = user_lock().clone();

        let user = block_on(
            async {
                user_lock_copies.lock().await
            }
        );

        oauth_url.set(user.slack.oauth_url.clone());
        
         // Clone the client if it exists (since we can't return a reference directly)
        if let Some(client) = mongo_client.as_ref() {
            // Convert the function into async and spawn it on the current runtime
            let client_clone = client.clone();  // Clone the client to avoid ownership issues
            let user_clone = user.clone();
            
            // Use `tokio::spawn` to run the async block
            block_on(async move {
                let db = client_clone.database(MONGO_DATABASE);
                let user_collection = db.collection::<User>(MONGO_COLLECTION);
                
                match to_bson(&user_clone.slack) {
                    Ok(slack_bson) => {
                        match user_collection
                            .find_one_and_update(
                                doc! { 
                                    "$or": [{"username": &user_clone.username}, 
                                            {"email": &user_clone.email}] 
                                },
                                doc! {
                                    "$set": { "slack": slack_bson }
                                }
                            )
                            .await 
                        {
                            Ok(Some(_)) => {
                                // Document found and updated
                                info!("Document updated successfully");
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
            class: format_args!("slack-login {}", if show_slack_login_pane() { "visible" } else { "" }),
            img {
                src: "assets/slack_logo.png",
                alt: "Slack Logo",
                width: "50px",
                height: "50px",
            }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| {show_slack_login_pane.set(false)},
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
           
            a {
                href: "https://api.slack.com/apps", // The URL you want to navigate to
                target: "_top",               // Opens in a new tab (optional)
                button { 
                    class: "login-button",
                    "Slack Api" 
                }
            }

            if logged_in() {
                a {
                    href: oauth_url().as_str(),  // The URL you want to navigate to
                    target: "_top",               // Opens in a new tab (optional)
                    button { 
                        class: "login-button",
                        onclick: move |_| {
                            current_platform.set("Slack".to_string());
                            show_slack_login_pane.set(false);
                        },
                        "Install Workspace" 
                    }
                }

            }
            else {
                button { 
                    class: "login-button",
                    onclick: handle_new_user, "Add WorkSpace" 
                }
            }


            // TODO: provide custom error warnings
            if let Some(error) = login_error() {
                
                p { 
                    style: "color: white; font-family: Arial, sans-serif; font-weight: bold; text-align: center;",
                    "{error}" 
                }
            }
        }
    }
}