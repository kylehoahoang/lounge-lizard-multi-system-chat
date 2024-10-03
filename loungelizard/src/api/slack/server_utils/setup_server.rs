
use slack_morphism::prelude::*;
use std::fs;
use url::Url;
use crate::api::slack::ngrok_s::*;
use dioxus_logger::tracing::{info, error, warn};
use crate::api::mongo_format::mongo_structs::*;


pub async fn update_server(user: User) {
    // Define the path to the JSON file
    let file_path = "src/api/slack/manifest/manifest.json";

    // Read the contents of the manifest file
    let manifest_file = fs::read_to_string(file_path).expect("Unable to read file");

    // Parse the manifest file into a `SlackAppManifest` struct
    let mut manifest_struct: SlackAppManifest = serde_json::from_str(&manifest_file).expect("Unable to parse JSON");

    // Create a new Slack client
    let client  = SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));

    let user_temp = user.clone(); 
    // Create a new token from the environment variable `SLACK_CONFIG_TOKEN`
    let token: SlackApiToken = SlackApiToken::new(user_temp.slack.config_token.into());

    // Create a new session with the client and the token
    let session = client.open_session(&token);

    let _ = ngrok_start_session("8080");

    let response = fetch_ngrok_tunnels().await.expect("failed");


    let mut redirect_url = String::new();
    // Extract the public URL from the first active tunnel
    if let Some(tunnels) = response.get("tunnels").and_then(|t| t.as_array()) {
        if let Some(tunnel) = tunnels.first() {
            if let Some(public_url) = tunnel.get("public_url").and_then(|url| url.as_str()) {
                // Set the environment variable `SLACK_REDIRECT_HOST` to the public URL
                redirect_url = public_url.to_string();
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
                *request_url = Url::parse(redirect_url.as_str()).expect("Failed to parse URL");
            }
        }
    }

    if let Some(ref mut oauth_config) = manifest_struct.oauth_config {
        if let Some(ref mut redir_urls) = oauth_config. redirect_urls{
        // Update the redirect URLs to contain only the public URL
        *redir_urls = vec![(Url::parse(redirect_url.as_str()).expect("Failed to parse URL"))];
        }
    }

    // Create a new app with the updated manifest
    let updated_app = SlackApiAppsManifestUpdateRequest::new(
        user.slack.app_id.clone().into(),
        manifest_struct.clone()
    );

    let _updated_response = 
        session.apps_manifest_update(&updated_app).await.expect("failed to update app");
}