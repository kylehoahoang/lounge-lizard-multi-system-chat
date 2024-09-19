extern crate dotenv;
use dotenv::dotenv;
use std::env; 
use std::fs;
use url::Url; 

use ::log::{debug, error, info};
use env_logger;

// ! Importing utils Transferable
mod utils; 
use utils::slack::ngrok_s::*;
use utils::slack::event_server::*;
use slack_morphism::prelude::*;
use utils::slack::config_env::config_env_var;


#[derive(Debug)]
struct UserStateExample(u64);



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Loads the environment variables from the `.env` file in the current working directory
    env_logger::init();
    
    // into the process's environment.
    dotenv().ok();

    // Define the path to the JSON file
    let file_path = "src/manifest/manifest.json";

    // Read the contents of the manifest file
    let manifest_file = fs::read_to_string(file_path).expect("Unable to read file");

    // Parse the manifest file into a `SlackAppManifest` struct
    let mut manifest_struct: SlackAppManifest = serde_json::from_str(&manifest_file).expect("Unable to parse JSON");

    // Create a new Slack client
    let client  = SlackClient::new(SlackClientHyperConnector::new()?);

    // Create a new token from the environment variable `SLACK_CONFIG_TOKEN`
    let token: SlackApiToken = SlackApiToken::new(config_env_var("SLACK_CONFIG_TOKEN")?.into());

    // Create a new session with the client and the token
    let session = client.open_session(&token);

    let _ = ngrok_start_session("8080");


    let response = fetch_ngrok_tunnels().await.expect("failed");

    // Extract the public URL from the first active tunnel
    if let Some(tunnels) = response.get("tunnels").and_then(|t| t.as_array()) {
        if let Some(tunnel) = tunnels.first() {
            if let Some(public_url) = tunnel.get("public_url").and_then(|url| url.as_str()) {
                // Set the environment variable `SLACK_REDIRECT_HOST` to the public URL
                env::set_var("SLACK_REDIRECT_HOST", public_url);
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
                *request_url = Url::parse(config_env_var("SLACK_REDIRECT_HOST")?.as_str())?;
            }
        }
    }

    if let Some(ref mut oauth_config) = manifest_struct.oauth_config {
        if let Some(ref mut redir_urls) = oauth_config. redirect_urls{
           // Update the redirect URLs to contain only the public URL
           *redir_urls = vec![(Url::parse(config_env_var("SLACK_REDIRECT_HOST")?.as_str())?)];
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
    env::set_var("SLACK_CLIENT_ID", created_app_reponse.credentials.client_id.to_string());
    env::set_var("SLACK_CLIENT_SECRET", created_app_reponse.credentials.client_secret.to_string());
    env::set_var("SLACK_VERIFICATION_TOKEN", created_app_reponse.credentials.verification_token.to_string());
    env::set_var("SLACK_SIGNING_SECRET", created_app_reponse.credentials.verification_token.to_string());
    env::set_var("SLACK_OAUTH_REDIRECT_URL", created_app_reponse.oauth_authorize_url.to_string());

    
    println!("OAUTH Redirect URL: {:?}", config_env_var("SLACK_OAUTH_REDIRECT_URL")?);
    
    events_api(manifest_struct).await?;

    Ok(())
}
