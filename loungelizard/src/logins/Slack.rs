use dioxus::prelude::*;
use bson::to_bson;
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

pub fn SlackLogin (show_slack_login_pane: Signal<bool>) -> Element {

    // ! User Mutex Lock to access the user data
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    let client_lock = use_context::<Signal<Arc<Mutex<Option<Client>>>>>();
    // ! ========================= ! //

    let mut logged_in = use_signal(||false);

    let mut login_error = use_signal(|| None::<String>);

    // ! Slack Temp Values ! //
    let mut client_id = use_signal(|| String::new());
    let mut client_secret = use_signal(|| String::new());
    let mut verif_token = use_signal(|| String::new());
    let mut signing_secret = use_signal(|| String::new());
    let mut redirect_host = use_signal(|| String::new());
    let mut oauth_url = use_signal(|| String::new());
    let mut config_token = use_signal(|| String::new());

    let handle_new_user = move |_| {

        let login = block_on(async move {
            // Collect Token from Clipboard
            let ctx = ClipboardContext::new().unwrap();
            config_token.set(ctx.get_text().unwrap_or("".to_string()));

            // Define the path to the JSON file
            let file_path = "src/api/slack/manifest/manifest.json";

            // Read the contents of the manifest file
            let manifest_file = fs::read_to_string(file_path).expect("Unable to read file");

            // Parse the manifest file into a `SlackAppManifest` struct
            let mut manifest_struct: SlackAppManifest = serde_json::from_str(&manifest_file).expect("Unable to parse JSON");

            // Create a new Slack client
            let client  = SlackClient::new(SlackClientHyperConnector::new()?);

            // Create a new token from the environment variable `SLACK_CONFIG_TOKEN`
            let token: SlackApiToken = SlackApiToken::new(config_token().into());

            // Create a new session with the client and the token
            let session = client.open_session(&token);

            let _ = ngrok_start_session("8080");

            let response = fetch_ngrok_tunnels().await.expect("failed");

            // Extract the public URL from the first active tunnel
            if let Some(tunnels) = response.get("tunnels").and_then(|t| t.as_array()) {
                if let Some(tunnel) = tunnels.first() {
                    if let Some(public_url) = tunnel.get("public_url").and_then(|url| url.as_str()) {
                        // Set the environment variable `SLACK_REDIRECT_HOST` to the public URL
                        redirect_host.set(public_url.to_string());
                    } else {
                        eprintln!("Public URL not found in the tunnel data.");
                    }
                } else {
                    eprintln!("No tunnels found.");
                }
            } else {
                eprintln!("Tunnels field not found in the response.");
            }

            // Update newly created URL into manifest befor creating the app
            if let Some(ref mut settings) = manifest_struct.settings {
                if let Some(ref mut event_subscriptions) = settings.event_subscriptions {
                    if let Some(ref mut request_url) = event_subscriptions.request_url {
                        // Update the request URL to the public URL
                        *request_url = Url::parse(redirect_host().as_str())?;
                    }
                }
            }

            if let Some(ref mut oauth_config) = manifest_struct.oauth_config {
                if let Some(ref mut redir_urls) = oauth_config. redirect_urls{
                // Update the redirect URLs to contain only the public URL
                *redir_urls = vec![(Url::parse(redirect_host().as_str())?)];
                }
            }

            // Create a new app with the updated manifest
            let new_app: SlackApiAppsManifestCreateRequest = SlackApiAppsManifestCreateRequest::new(
                SlackAppId::new("-".into()),
                manifest_struct.clone()
            );

            // Create the app
            let created_app_reponse: SlackApiAppsManifestCreateResponse = session.apps_manifest_create(&new_app).await?;

            // Set Env vars without manually inputting them 
            client_id.set(created_app_reponse.credentials.client_id.to_string());
            client_secret.set(created_app_reponse.credentials.client_secret.to_string());
            verif_token.set(created_app_reponse.credentials.verification_token.to_string());    
            signing_secret.set(created_app_reponse.credentials.verification_token.to_string());
            oauth_url.set(created_app_reponse.oauth_authorize_url.to_string());

            // Since we are returning a Result, specify the Ok variant explicitly
            Ok::<(), Box<dyn std::error::Error>>(())
        });

         // Handle the result of the login operation
        match login {
            Ok(_) => {
                println!("Login successful!");
                logged_in.set(true);
                // Continue with the rest of your program logic here
            }
            Err(e) => {
                error!("Login failed: {}", e);
                return;
                // Handle the error, possibly exit or retry
            }
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
            user.slack = Slack {
                app_id: "".to_string(),
                bot: Bot {
                    token: "".to_string(),
                    scope: "".to_string(),
                },
                client_id: client_id().to_string(),
                client_secret: client_secret().to_string(),
                config_token: config_token().to_string(),
                oauth_url: oauth_url().to_string(),
                team: Team {
                    id: "".to_string(),
                    name: "".to_string(),
                },
                user: Slack_User {
                    token: "".to_string(),
                    scope: "".to_string(),
                },
                verif_token: verif_token().to_string(),
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
                            let navigator = use_navigator();
                            navigator.push(AppRoute::Slack{});
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
                    "Login failed: {error}" 
                }
            }
        }
    }
}