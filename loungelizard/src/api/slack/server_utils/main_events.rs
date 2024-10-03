

use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::{Method,Request, Response, StatusCode};
use serde_json::Value;
use tokio::time::{self, Duration};
use std::sync::{Arc};
use tokio::sync::Mutex;
use std::convert::Infallible;
use std::env::{self, set_var};

use url::form_urlencoded;
use dioxus_logger::tracing::{debug, error, warn, info, Level};
use slack_morphism::prelude::*;
use crate::api::slack::config_env::config_env_var;
use crate::api::slack::server_utils::request_server::request_server;
use crate::api::mongo_format::mongo_structs::*;
// Import the necessary error handling types
// Define the global queue as a Mutex wrapped around a vector of QueueItem
lazy_static::lazy_static! {
    static ref GLOBAL_QUEUE: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    static ref TEMP_CODE: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}


/// Consume incoming requests from the global queue.
///
/// This function is an infinite loop that takes requests from the global queue
/// and processes them. If there are no requests waiting, it will sleep for a
/// second before checking again.
///
/// This function is called in an infinite loop in `main_events`.
pub async fn request_consumer(
    mut user: &mut User
    // TODO Intake an instance or modifiable of the UI
)-> Result<Response<BoxBody<Bytes, Infallible>>, Box<dyn std::error::Error + Send + Sync>> {
    // This is a loop that will run indefinitely
    // Its purpose is to process any incoming requests we have
    loop {
        // First, we need to check if there are any requests waiting
        // We do this by locking the queue and checking its length
        // If there are requests waiting, we pop the last one from the queue
        // If there are no requests waiting, we just skip to the end of the loop
        let request = {
            let mut queue = GLOBAL_QUEUE.lock().await;
            // If there are no requests waiting, we just bail out
            if queue.is_empty() {
                None
            }
            // Otherwise, we pop the last request from the queue
            else {
                Some(queue.pop().unwrap())
            }
        };

        // If there was a request waiting, we process it
        if let Some(request) = request {
            // We call request_server with the request as an argument
            // This will do something with the request, like respond to it
            // TODO Pass in an instance of the UI
            
            request_server(request).await;
        }
        // If there were no requests waiting, we just sleep for a second
        else {
            // Sleep for a second before checking again
            time::sleep(Duration::from_millis(1)).await;
        }

        // Will only be invoked for installtion
        let mut temp_code_lock = TEMP_CODE.lock().await;

        if let Some(code) = &*temp_code_lock {
            info!("Code received: {}", code);
            
            let client  = SlackClient::new(SlackClientHyperConnector::new()?);

            // We need to handle the installation request
            let temp_code = SlackOAuthCode::new(code.to_string());

            let oauth_token_request = SlackOAuthV2AccessTokenRequest::new(
                user.slack.client_id.clone().into(),
                user.slack.client_secret.clone().into(),
                temp_code,
            );

            // use the client variable here
            let oauth_response: Result<SlackOAuthV2AccessTokenResponse, errors::SlackClientError> = 
                client.oauth2_access(&oauth_token_request).await;

            match oauth_response {
                Ok(oauth_response) => {
                    user.slack.app_id       = oauth_response.app_id.to_string();
                    user.slack.bot.token    = oauth_response.access_token.to_string();
                    user.slack.bot.scope    = oauth_response.scope.to_string();
                    user.slack.team.id      = oauth_response.team.id.to_string();
                    user.slack.team.name    = oauth_response.team.name.unwrap().to_string();
                    user.slack.user.token   = oauth_response.authed_user.access_token.unwrap().to_string();
                    user.slack.user.scope   = oauth_response.authed_user.scope.unwrap().to_string();
                    //user.slack.user.      = oauth_response.authed_user.id.to_string();
            
                }
                Err(e) => {
                    error!("Error: {:#?}", e);
                }
            }

            *temp_code_lock = None;
        }

        
    }
}
// 

pub async fn main_event_api(  
        req: Request<Incoming>,
    ) -> Result<Response<BoxBody<Bytes, Infallible>>, Box<dyn std::error::Error + Send + Sync>>
    {
        // Check the HTTP method of the request
        match req.method() {
            // If it's a POST request
            &Method::POST => {
                // We have received a POST request, which means we have received an event from Slack
                
                // We need to collect the body of the request
                let whole_body = req.collect().await?.to_bytes();

                // We need to parse the body as a JSON string
                let byte_string = String::from_utf8_lossy(&whole_body);
                let json_value: Value = serde_json::from_str(&byte_string).unwrap();

                // We need to check if the event is a challenge or not
                if let Value::Object(ref map) = json_value {
                    // If the event is a challenge, we need to respond with the challenge
                    if let Some(challenge) = map.get("challenge").and_then(Value::as_str).map(|s| s.to_string()){
                        debug!("Challenge received, responding with challenge");

                        // We respond with the challenge
                        return Response::builder()
                            .status(200)
                            .header("Content-type", "text/plain")
                            .body(Full::new(challenge.into()).boxed())
                            .map_err(|e| e.into());
                    }
                    else {
                        // We add the event to the queue
                        GLOBAL_QUEUE.lock().await.push(json_value); 
                    }
                }
                else {
                    println!("Unknown event type");
                }
                
                // We return a response
                Response::builder()
                    .status(200)
                    .body(Full::new("".into()).boxed())
                    .map_err(|e| e.into())
            }
            // If it's a GET request
            &Method::GET => {
                // We have received a GET request, which means we have received an installation request

                // We need to parse the query string into a HashMap
                let body_string = req.uri().query().unwrap_or("").to_string();

                let parsed_query = form_urlencoded::parse(body_string.as_bytes())
                .into_owned()
                .collect::<std::collections::HashMap<_, _>>();

                // Assuming `parsed_query` is a `HashMap` or similar structure
                if let Some(code) = parsed_query.get("code") {
                   
                     // Acquire a lock on the Mutex
                    let mut temp_code_lock = TEMP_CODE.lock().await;
                    
                    // Update the Option<String> inside the Mutex
                    *temp_code_lock = Some(code.to_string());
                    
                } else {
                    warn!("Unknown GET request");
                }
                
                // Handle GET requests
                Response::builder()
                    .body(Full::new("You may now close this window".into()).boxed())
                    .map_err(|e| e.into())
            }
            // Handle other HTTP methods
            _ => {
                // Return a 405 Method Not Allowed response for unsupported methods
                Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .body(Full::new("Method Not Allowed".into()).boxed())
                    .map_err(|e| e.into())
            }
        }
    }
