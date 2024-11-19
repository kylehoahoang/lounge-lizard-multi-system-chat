use reqwest::header::{CONTENT_TYPE};
use serde_json::Value;
use std::process::Command;
use tokio::sync::oneshot;
use std::collections::HashMap;
use std::process::Stdio;
use std::error::Error;
use which::which;
use tiny_http::{Server, Response};
use rand::Rng;
use sha2::{Digest, Sha256};
use base64::{encode_config, URL_SAFE_NO_PAD};
//use webbrowser;

// Constants 
const NGROK_TUNNEL_SEARCH: &str = "http://127.0.0.1:4040/api/tunnels";

/*
    Run MS Teams app setup code in the correct order

    Description: Starts the process of running ngrok, getting the ngrok uri
    updating the MS Azure manifest, authorizing a user, and getting an
    access token. More information on each aspect found in their respective
    function.

    Arguments: The client_id (ms_teams_client_id: &str) for the api stored
    as a constant, which will stay the same unless under new dev

    Returns: The access token used to interact with ms teams (String)
*/

pub async fn start_ms_teams(ms_teams_client_id: &str) -> Result<String, Box<dyn Error>> {

    let port = "8080";
    
    //let _ = start_ngrok(&port).await;
    let redirect_uri = fetch_ngrok_tunnel().await?;
    let _ = update_manifest_uri(ms_teams_client_id, &redirect_uri).await;
        
    //now get auth/access
    let (code_verifier, code_challenge) = generate_pkce_pair();
    let auth_code = get_authorization(ms_teams_client_id, &redirect_uri, &code_challenge).await?;
    let access_token = get_access_token(ms_teams_client_id, &redirect_uri, &auth_code, &code_verifier).await?;
    Ok(access_token)
}
/*
    Get authorization code from MS_Teams

    Command: chrome --incognito 'auth_url'

    Description: Open up a browser to allow users to log in, which will then
    send back an authorization code to our redirect_uri that will be automatically
    retrieved by our local server and used for getting an access code. Note that
    this authorization code needs a PCKE pair (proof key for code exchange).
    See generate_pcke_pair() for more info.

    Arguments: The client_id (client_id: &str), the ngrok tunnel callback uri
    used (redirect_uri: &str), and a PCKE code challenge (code_challenge: &str)

    Returns: The authorization code needed for an access token (String)
*/
async fn get_authorization(client_id: &str, redirect_uri: &str, code_challenge: &str) -> Result<String, Box<dyn Error>> {
    
    let endpoint = "https://login.microsoftonline.com/organizations/oauth2/v2.0/authorize";
    
    //will need to grab client_id from mongoDB
    //let client_id: &str = &env::var("CLIENT_ID").expect("client_id not set");

    let auth_url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope=offline_access%20https://graph.microsoft.com/.default&code_challenge={}&code_challenge_method=S256&state=12345",
        endpoint, client_id, redirect_uri, code_challenge
    );

    //setup a one time channel to receive the authorization code
    let (tx, rx) = oneshot::channel();
    tokio::task::spawn_blocking(move || {
        println!("Starting local server...");
        let _ = start_local_server(tx);
    });

    /* 
        Includes two options for logging in users. Webbrowser is a little simpler,
        but lacks versatility. Command runs a command function, but needs chrome.exe
        in the PATH. The benefit of using chrome for our purposes is it prevents
        Microsoft's automatic logging in, good for development purposes using
        different users on one machine.

        webbrowser::open(&auth_url).unwrap();
    */

    Command::new("chrome")
        .arg("--incognito")
        .arg(&auth_url)
        .spawn()
        .expect("Failed to open browser in incognito mode");

    let authorization_code = rx.await.expect("Failed to receive authorization code");
    Ok(authorization_code)
}

/*
    Generate a PCKE (proof key for code exchange) pair for auth/access

    Description: Generate an identifier to be used for authorization and access.
    This means we will include a SHA256 encoded code_challenge along an authorization
    request which will be somewhere in the resulting authorization code. This will be
    verified by the decoded code_verifier included in a request for an access code.

    Arguments: N/A

    Returns: A pair of strings, one is a random string of chars (code_verifier: String),
    the other is an encoded version of that string (code_challenge: String)
*/
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

/*
    Send a request to MS Graphs for a delegated Access Token

    Endpoint: https://login.microsoft.com/organizations/oauth2/token

    Description: After receiving authorization, request for a delegated
    access token (essentially a token that only the authorized user can
    use). This token has permissions to interact with all relevant
    aspects of MS Teams. Note that this access code needs a PCKE pair 
    (proof key for code exchange). See generate_pcke_pair() for more info.

    Arguments: Requires the client_id (client_id: &str), the redirect_uri that 
    received the authorization code(redirect_uri: &str), the authorization code
    (auth_code: &str) and a code verifier to authenticate client (code_verifier: &str)

    Returns: The delegated Access Token used for interacting with MS Teams (String)
*/

//start process for getting access token, including getting authorization
async fn get_access_token(client_id: &str, redirect_uri: &str, auth_code: &str, code_verifier: &str) -> Result<String, Box<dyn Error>> {

    //endpoint url
    let token_url = "https://login.microsoft.com/organizations/oauth2/token";

    //json form, including all info for access code
    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("redirect_uri", redirect_uri);
    params.insert("grant_type", "authorization_code");
    params.insert("code", auth_code);
    params.insert("code_verifier", code_verifier);

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

/*
    Start a local server to capture the authorization code

    Description: This function initiates a simple HTTP server on a local address
    and port. The server listens for a redirect request to '/callback' and parses
    the authorization code from the formatted response. This is then sent to the
    tx Sender object to be used there.

    Arguments: A channel sender for passing the extracted authorization code (tx: Sender<String>)

    Returns: N/A

*/
fn start_local_server(tx: oneshot::Sender<String>) -> Result<(), Box<dyn Error + Send + Sync>> {

    let server = Server::http("127.0.0.1:8000")?;
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
            request.respond(response)?;
        }
    }
    Ok(())
}

/*
    Start ngrok tunnel.

    Command: ngrok http 'local_url'

    Description: Starts an ngrok tunnel on the given port. This essentially
    just runs a command programmatically so that we can receive information
    more freely through our local server, since MS Teams doesn't want to 
    send information to a local server normally.

    Arguments: Some port number designated for a ngrok tunnel (port: &str)

    Returns: N/A

*/
async fn start_ngrok(port: &str) -> Result<(), Box<dyn Error>> {
    let local_url = format!("http://localhost:{}", port);

    let ngrok_path = which("ngrok").expect("Ngrok (ngrok) not found. Please ensure it is installed and available in PATH.");

    let ngrok_process = Command::new(ngrok_path)
        .arg("http")
        .arg(local_url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    
    if let Err(err) = ngrok_process {
        eprintln!("Failed to update Azure Manifest: {}", err);
        return Err(Box::new(err))
    }

    Ok(())
}

/*
    Fetch ngrok tunnel.

    Endpoint: 127.0.0.1:4040

    Description: Port 4040 shows information on connections, including Ngrok.
    By doing a get request at this endpoint, we can determine how many tunnels
    and public_urls are connected to our local machine. I.e. we can grab the 
    ngrok url generated in order to use it to update the manifest.

    Arguments: N/A

    Returns: ngrok tunnel's url appended with /callback (String)
*/
async fn fetch_ngrok_tunnel() -> Result<String, Box<dyn Error>> {
    let response = reqwest::get(NGROK_TUNNEL_SEARCH).await.unwrap();
    let json: Value = response.json::<Value>().await.unwrap();

    let tunnels = json.get("tunnels").and_then(|t| t.as_array()).unwrap();
    let tunnel = tunnels.get(0).unwrap();
    let public_url = tunnel.get("public_url").and_then(|p| p.as_str()).unwrap();
    let formatted_url = format!("{}/callback", public_url);

    Ok(formatted_url)
}

/*
    Update Azure Manifest.

    Command: az ad app update --id 'client_id' --public-client-redirect-uris 'new_redirect_uri'

    Description: We can update a pulic client's manifest easily using Azure CLI.
    The command can be used, using client_id for identification of which manifest,
    and the redirect_uri to change acceptable redirects. This will be used for
    authorization, as it needs to redirect to a place where we can obtain the code.

    Arguments: The client_id (client_id: &str), and a uri that can be easily accessed
    by your machine (new_redirect_uri: &str)

    Returns: N/A
*/
async fn update_manifest_uri(client_id: &str, new_redirect_uri: &str) -> Result<(), Box<dyn Error>> {

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
        return Err(Box::new(err))
    }
    Ok(())
}

/*
    Check token validity.

    Endpoint: https://graph.microsoft.com/v1.0/me

    Description: This is just a dummy check to see if a token
    is already valid. If this is successful, then we can skip
    the regular log in process.

    Arguments: A valid/invalid delegated access token (token: &str)

    Returns: Returns if the access token is valid (bool)
*/
pub async fn dummy_token_check(token: &str) -> bool {
    let client = reqwest::Client::new();
    let url = "https://graph.microsoft.com/v1.0/me";

    let response = client
        .get(url)
        .bearer_auth(token)
        .send()
        .await;
    
    match response {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}