//src/ngrok_s.rs
// NGROK Utilities for the Slack Lounge Lizard workspace
use std::process::{Command, Child, Stdio};
use std::io;

use dioxus_logger::tracing::{info, error, warn, Level};
use reqwest::Error;
use serde_json::Value;

// ! Constants for URL Paths 
/// *******************************************************
const NGROK_TUNNEL_SEARCH: &str = "http://127.0.0.1:4040/api/tunnels";

/// *******************************************************
/// 
/// ? NGROK Utilities for the Slack Lounge Lizard workspace
/// 
/// *******************************************************

/// Get the URL that ngrok is listening on
///
/// # Arguments
///
/// * `port`: The port to which ngrok should forward requests
///
/// # Returns
///
/// A string representing the URL that ngrok is listening on
pub fn ngrok_getURL(port: &str) -> String {
    format!("http://localhost:{}", port)
}

/// Starts an ngrok session
///
/// The function starts an ngrok process and forwards requests from the public ngrok.io URL
/// to the given port on localhost. This is useful for testing webhooks in a development
/// environment.
///
/// # Arguments
///
/// * `port`: The port to which ngrok should forward requests
///
/// # Return
///
/// The function returns the handle to the started ngrok process.
///
/// # Errors
///
/// The function returns an error if the ngrok process could not be started.
pub fn ngrok_start_session(port: &str) -> io::Result<Child> {
    
    info!("Starting ngrok session");

    let local_url = format!("http://localhost:{}", port);

    let ngrok_process = Command::new("ngrok")
        .arg("http")
        .arg(local_url) // Expose port 8080 to the public
        .stdout(Stdio::null()) // Redirect output to null to avoid clutter
        .stderr(Stdio::null()) // Redirect errors to null
        .spawn(); // Spawn the process in the background


    match ngrok_process {
        Ok(child) => Ok(child),
        Err(err) => 
        {
            error!("Failed to start ngrok: {}", err);
            Err(err)
        }
    }
}

/// Kills an existing ngrok session.
///
/// The function takes a mutable reference to the ngrok process and kills it.
///
/// # Arguments
///
/// * `child`: A mutable reference to the ngrok process handle.
///
/// # Return
///
/// The function returns a `Result` indicating whether the process was killed
/// successfully. If the process was already dead, the function returns an error.
pub fn ngrok_kill_session(child: &mut Child) -> io::Result<()> {
    info!("Killing ngrok session");

    // Attempt to kill the process and log any error
    match child.kill() {
        Ok(_) => {
            info!("Successfully killed ngrok session.");
        }
        Err(e) => {
            // Log the error if killing the process fails
            error!("Failed to kill ngrok session: {}", e);
            return Err(e); // Return the error to the caller
        }
    }

    Ok(())

}

/// Sends an HTTP GET request to the ngrok API at the URL specified in the `NGROK_TUNNEL_SEARCH` constant.
///
/// The function waits for the response to arrive and then attempts to parse the response body as JSON.
///
/// # Returns
///
/// If the request is successful and the response body can be parsed as JSON, the function returns the parsed JSON `Value`.
/// If any error occurs, the function returns an `Error`.
pub async fn fetch_ngrok_tunnels() -> Result<Value, Error> {
    // Send a GET request to the ngrok API at the URL specified in the `NGROK_TUNNEL_SEARCH` constant
    let response = reqwest::get(NGROK_TUNNEL_SEARCH)
        .await?;

    // Attempt to parse the response body as JSON
    let json_value = response
        .json::<Value>()
        .await?;

    // If the JSON parsing was successful, return the parsed JSON `Value`
    Ok(json_value)
}


// ! Unit Level Testing

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ngrok_start_session() {
        let child = ngrok_start_session("8080").unwrap();
        assert!(child.id() > 0);
    }

    #[tokio::test]
    async fn test_ngrok_kill_session() {
        let mut child = ngrok_start_session("8080").unwrap();
        ngrok_kill_session(&mut child).unwrap();
        assert!(child.try_wait().unwrap().is_none());
    }

    #[tokio::test]
    async fn test_ngrok_getURL() {
        let url = ngrok_getURL("8080");
        assert_eq!(url, "http://localhost:8080");
    }

    #[tokio::test]
    async fn test_fetch_ngrok_tunnels() {
        let json_value = fetch_ngrok_tunnels().await.unwrap();
        assert!(json_value.is_object());
    }
}

