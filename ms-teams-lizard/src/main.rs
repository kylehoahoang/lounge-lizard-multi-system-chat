use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use dotenv::dotenv;
use std::env;
use std::io::{self};
use std::collections::HashMap;
use tiny_http::{Server, Response};
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use rand::Rng;
use sha2::{Digest, Sha256};
use base64::{encode_config, URL_SAFE_NO_PAD};
use std::process::{Command, Stdio};
use which::which;
use tokio::time::{sleep, Duration};
use webbrowser;

//testing main that will both send and receive messages from a specific channel
#[tokio::main]
async fn main() {
    load_config();
    let redirect_uri = start_ngrok().await;
    //may consider waiting a short duration because of update delays, should be able to avoid
    //sleep(Duration::from_secs(45)).await;
    match get_access_token(&redirect_uri).await {
        Ok(access_token) => {
            match get_teams(&access_token).await {
                Ok(teams_list) => {
                    if teams_list.is_empty() {
                        println!("No teams found.");
                    }
                    else {
                        println!("Select a Team:");
                        for (index, (id, name)) in teams_list.iter().enumerate() {
                            println!("{}: Team Name: {}, Team ID: {}", index + 1, name, id);
                        }
                        let mut selection = String::new();
                        println!("Enter the number in front of the Team you want to select:");
                        io::stdin()
                            .read_line(&mut selection)
                            .expect("Failed to read line");
                        let selection: usize = selection.trim().parse().expect("Please enter a valid number");
                        if selection > 0 && selection <= teams_list.len() {
                            let (selected_team_id, selected_team_name) = &teams_list[selection - 1];
                            println!("You selected Team: {}, ID: {}", selected_team_name, selected_team_id);

                            match get_channels(&access_token, &selected_team_id).await {
                                Ok(channels_list) => {
                                    if channels_list.is_empty() {
                                        println!("No channels found.");
                                    }
                                    else {
                                        println!("Select a Channel:");
                                        println!("Channels in Team {}:", selected_team_name);
                                        for (index, (id, name)) in channels_list.iter().enumerate() {
                                            println!("{}: Channel Name: {}, Channel ID: {}", index + 1, name, id);
                                        }
                                        let mut selection = String::new();
                                        println!("Enter the number in front of the Channel you want to select:");
                                        io::stdin()
                                            .read_line(&mut selection)
                                            .expect("Failed to read line");
                                        let selection: usize = selection.trim().parse().expect("Please enter a valid number");
                                        if selection > 0 && selection <= channels_list.len() {
                                            let (selected_channel_id, selected_channel_name) = &channels_list[selection - 1];
                                            println!("You selected Channel: {}, ID: {}", selected_channel_name, selected_channel_id);
                                            //now send a message
                                            let message = "Hello world! :)";
                                            match send_message(&access_token, message, selected_team_id, selected_channel_id).await {
                                                Ok(_) => println!("Message sent successfully!"),
                                                Err(err) => eprintln!("Failed to send message: {}", err),
                                            }
                                            match get_messages(&access_token, selected_team_id, selected_channel_id).await {
                                                Ok(messages_list) => {
                                                    if messages_list.is_empty() {
                                                        println!("No messages sent in channel.");
                                                    }
                                                    else {
                                                        println!("Messages in Channel ({}):", selected_channel_name);
                                                        for (id, content, sender) in messages_list {
                                                            println!("Message ID: {}\nSender: {}\nContent: {}\n---", id, sender, content);
                                                        }
                                                    }
                                                }
                                                Err(err) => {
                                                    eprintln!("Failed to retrieve messages: {}", err);
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("Error fetching channels: {:?}", e);
                                }
                            }
                        }

                    }
                }
                Err(e) => {
                    println!("Error fetching teams: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("failed to get access token: {:?}", e);
        }
    }
}

//retrieve teams, given delegated access token
async fn get_teams(access_token: &str) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let url = "https://graph.microsoft.com/v1.0/me/joinedTeams";

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        let teams: Value = response.json().await?;
        
        let mut teams_list = Vec::new();

        if let Some(values) = teams.get("value").and_then(|v| v.as_array()) {
            for team in values {
                if let (Some(id), Some(display_name)) = (
                    team.get("id").and_then(|v| v.as_str()),
                    team.get("displayName").and_then(|v| v.as_str()),
                ) {
                    teams_list.push((id.to_string(), display_name.to_string()));
                }
            }
        }
        Ok(teams_list)
    }
    else {
        Err(format!("Failed to retrieve teams: {}", response.status()).into())
    }
}

//retrieve channels given team_id
async fn get_channels(access_token: &str, team_id: &str) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels", team_id);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        let channels: Value = response.json().await?;
        let mut channels_list = Vec::new();
        if let Some(values) = channels.get("value").and_then(|v| v.as_array()) {
            for channel in values {
                if let (Some(id), Some(display_name)) = (
                    channel.get("id").and_then(|v| v.as_str()),
                    channel.get("displayName").and_then(|v| v.as_str()),
                ) {
                    channels_list.push((id.to_string(), display_name.to_string()));
                }
            }
        }
        Ok(channels_list)
    }
    else {
        Err(format!("Failed to retrieve channels: {}", response.status()).into())
    }
}

//retrieve messages given team_id & channel_id
async fn get_messages(access_token: &str, team_id: &str, channel_id: &str) -> Result<Vec<(String, String, String)>, Box<dyn std::error::Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels/{}/messages", team_id, channel_id);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        let messages: Value = response.json().await?;
        let mut messages_list = Vec::new();

        if let Some(values) = messages.get("value").and_then(|v| v.as_array()) {
            for message in values {
                let id = message.get("id").and_then(|v| v.as_str()).unwrap_or("Unknown ID").to_string();
                let content = message
                    .get("body")
                    .and_then(|b| b.get("content"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("No content")
                    .to_string();
                let sender = message
                    .get("from")
                    .and_then(|f| f.get("user"))
                    .and_then(|u| u.get("displayName"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown sender")
                    .to_string();

                messages_list.push((id, content, sender));
            }
        }
        Ok(messages_list)
    }
    else {
        let status = response.status();
        let error_body = response.text().await?;
        eprintln!("Failed to retrieve messages: HTTP {} - {}", status, error_body);
        Err(format!("Failed to retrieve messages: HTTP {} - {}", status, error_body).into())
    }
}

//send some given string to ms teams
async fn send_message(access_token: &str, message: &str, team_id: &str, channel_id: &str) -> Result<(), Box<dyn std::error::Error>> {

    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels/{}/messages", team_id, channel_id);

    //this can probably be modified to send different files, emojis, etc
    let body = json!({
        "body": {
            "content": message
        }
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await?;
        eprintln!("Failed to send message: HTTP {} - {}", status, error_body);
    }
    Ok(())
}

//start process for getting access token, including getting authorization
async fn get_access_token(redirect_uri: &str) -> Result<String, Box<dyn std::error::Error>> {
    
    //access necessary variables
    let client_id: &str = &env::var("CLIENT_ID").expect("client_id not set");
    //let redirect_uri: &str = &env::var("REDIRECT_URI").expect("redirect_uri not set");

    //setup a one time channel to receive the authorization code
    let (tx, rx) = oneshot::channel();
    tokio::task::spawn_blocking(move || {
        println!("Starting local server...");
        start_local_server(tx);
    });

    //get the authorization code and a verification code to pair this auth with access code post
    let code_verifier= get_authorization(redirect_uri);
    let authorization_code = rx.await.unwrap();
    println!("Authorization received");

    //endpoint url
    let token_url = "https://login.microsoft.com/organizations/oauth2/token";

    //json form, including all info for access code
    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("redirect_uri", redirect_uri);
    params.insert("grant_type", "authorization_code");
    params.insert("code", &authorization_code);
    params.insert("code_verifier", &code_verifier);

    let client = reqwest::Client::new();
    let response = client
        .post(token_url)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await?;
        return Err(format!("Failed to get access token: HTTP {} - {}", status, error_body).into());
    }

    let token_response: HashMap<String, String> = response.json().await?;
    if let Some(access_token) = token_response.get("access_token") {
        Ok(access_token.clone())
    }
    else {
        Err("Access token not found in the response".into())
    }
}

//get authorization by opening browser and logging user in, retrieve redirected auth code using ngrok
fn get_authorization(redirect_uri: &str) -> String {
    let url = "https://login.microsoftonline.com/organizations/oauth2/v2.0/authorize";
    let client_id: &str = &env::var("CLIENT_ID").expect("client_id not set");

    let (code_verifier, code_challenge) = generate_pkce_pair();

    let final_url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope=offline_access%20https://graph.microsoft.com/.default&code_challenge={}&code_challenge_method=S256&state=12345",
        url, client_id, redirect_uri, code_challenge
    );

    //webbrowser::open(&final_url).unwrap();

    //use this to force incognito mode and disallow auto log in
    Command::new("chrome")
        .arg("--incognito")
        .arg(&final_url)
        .spawn()
        .expect("Failed to open browser in incognito mode");

    code_verifier
}

//generate an identifier where one can be sent to authorization req and the other to access token req to verify app
fn generate_pkce_pair() -> (String, String) {
    let code_verifier: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();

    let hash = Sha256::digest(code_verifier.as_bytes());
    let code_challenge = encode_config(hash, URL_SAFE_NO_PAD);

    (code_verifier, code_challenge)
}

//start a local server to get the authorization code from callback
fn start_local_server(tx: Sender<String>) {

    let server = Server::http("127.0.0.1:8000").expect("Could not start server on port 8000");
    //handle one callback
    if let Some(request) = server.incoming_requests().next() {
        let url = request.url().to_string();
        if url.starts_with("/callback") {
            if let Some(query) = url.split('?').nth(1) {
                for param in query.split('&') {
                    let mut key_val = param.split('=');
                    if let (Some(key), Some(value)) = (key_val.next(), key_val.next()) {
                        if key == "code" {
                            let _ = tx.send(value.to_string());
                            break;
                        }
                    }
                }
            }
            let response = Response::from_string("Authorization successful! You can close this window.");
            let _ = request.respond(response);
        }
    }
}

//setup .env file, this is temporary
fn load_config() {
    dotenv::from_path("./tokens.env").expect("Failed to read .env file");
    dotenv().ok();
}

const NGROK_TUNNEL_SEARCH: &str = "http://127.0.0.1:4040/api/tunnels";

// Function to start ngrok
pub async fn start_ngrok() -> String {
    // Set up the ngrok server to forward to the local URL
    let local_url = "http://localhost:8000";

    let ngrok_path = which("ngrok").expect("Ngrok (ngrok) not found. Please ensure it is installed and available in PATH.");

    let ngrok_process = Command::new(ngrok_path)
        .arg("http")
        .arg(local_url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

        let tunnel_url = fetch_ngrok_tunnel().await;
        println!("New Ngrok URL: {}", tunnel_url);
        update_manifest_url(&tunnel_url).await;

    tunnel_url
}

// Function to fetch ngrok tunnel
pub async fn fetch_ngrok_tunnel() -> String {
    let response = reqwest::get(NGROK_TUNNEL_SEARCH).await.unwrap();
    let json = response.json::<Value>().await.unwrap();

    let tunnels = json.get("tunnels").and_then(|t| t.as_array()).unwrap();
    let tunnel = tunnels.get(0).unwrap();
    let public_url = tunnel.get("public_url").and_then(|p| p.as_str()).unwrap();
    let formatted_url = format!("{}/callback", public_url);

    formatted_url
}

// Function to update Azure manifest
pub async fn update_manifest_url(new_redirect_uri: &str) {
    let client_id: &str = &env::var("CLIENT_ID").expect("client_id not set");

    let az_cli_path = which("az").expect("Azure CLI (az) not found. Please ensure it is installed and available in PATH.");

    let update_command = Command::new(az_cli_path)
        .arg("ad")
        .arg("app")
        .arg("update")
        .arg("--id")
        .arg(client_id)
        .arg("--public-client-redirect-uris")
        .arg(new_redirect_uri)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap()
        .wait();

    if let Err(err) = update_command {
        eprintln!("Failed to update Azure Manifest: {}", err);
    }
}
