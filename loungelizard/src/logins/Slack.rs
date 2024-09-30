use dioxus::prelude::*;
use bson::to_bson;
use crate::{AppRoute};

use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};
use crate::api::mongo_format::mongo_structs::*;
use crate::api::mongo_format::mongo_funcs::*;   
use dioxus_logger::tracing::{info, error, warn};
use futures::executor::block_on;
use mongodb::{sync::Client, bson::doc};

use std::sync::{Arc, Mutex};

#[component]
pub fn SlackLogin (show_slack_login_pane: Signal<bool>) -> Element {

    // ! User Mutex Lock to access the user data
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    let client_lock = use_context::<Signal<Arc<Mutex<Option<Client>>>>>();
    // ! ========================= ! //

    let mut logged_in = use_signal(|| false);
    let mut login_error = use_signal(|| None::<String>);

    let handle_new_user = move |_| {
        // let ctx = ClipboardContext::new().unwrap();
        // let token = ctx.get_text().unwrap_or("".to_string());

        // TODO: Implement slack login code
        if !login_error().is_none() {return;}

        let mongo_lock_copies = client_lock().clone();
        let user_lock_copies = user_lock().clone();

        let mongo_client = mongo_lock_copies.lock().unwrap();
        let mut user = user_lock_copies.lock().unwrap();

         // Clone the client if it exists (since we can't return a reference directly)
        if let Some(client) = mongo_client.as_ref() {
            // Convert the function into async and spawn it on the current runtime
            let client_clone = client.clone();  // Clone the client to avoid ownership issues

            // TODO Add all tokens to user profile here
            user.slack = Slack {
                app_id: "testId".to_string(),
                bot: Bot {
                    token: "".to_string(),
                    scope: "".to_string(),
                },
                client_id: "".to_string(),
                client_secret: "".to_string(),
                config_token: "".to_string(),
                oauth_url: "".to_string(),
                redirect_host: "".to_string(),
                team: Team {
                    id: "".to_string(),
                    name: "".to_string(),
                },
                user: SlackUser {
                    token: "".to_string(),
                    scope: "".to_string(),
                },
                verif_token: "".to_string(),
            };
            
            // Todo ====================================//

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
                                logged_in.set(true);
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
            Link { 
                to: if logged_in() {AppRoute::Slack {}} else {AppRoute::Home {}},
                button { 
                    class: "login-button",
                    onclick: handle_new_user, "Add WorkSpace" 
                }
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