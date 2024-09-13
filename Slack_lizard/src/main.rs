extern crate dotenv;
use slack_morphism::prelude::*;

use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Method,Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tracing::*;
use serde_json::Value;
use dotenv::dotenv;
use std::env; 
use tokio::time::{self, Duration};
use std::sync::{Arc, Mutex};
use std::convert::Infallible;
use lazy_static::lazy_static;
use std::fs;
use std::process::{Command, Stdio};

use url::Url; 
use url::form_urlencoded;


async fn test_push_events_function(
    event: SlackPushEvent,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read state
    let current_state = {
        let states = _states.read().await;
        println!("{:#?}", states.get_user_state::<UserStateExample>());
        println!("{:#?}", states.len());
        UserStateExample(states.get_user_state::<UserStateExample>().unwrap().0 + 1)
    };

    // Write state
    {
        let mut states = _states.write().await;
        states.set_user_state::<UserStateExample>(current_state);
        println!("{:#?}", states.get_user_state::<UserStateExample>());
    }

    println!("{:#?}", event);
    Ok(())
}

async fn test_interaction_events_function(
    event: SlackInteractionEvent,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Interevent {:#?}", event);
    Ok(())
}

async fn test_command_events_function(
    event: SlackCommandEvent,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<SlackCommandEventResponse, Box<dyn std::error::Error + Send + Sync>> {
    let token_value: SlackApiTokenValue = config_env_var("SLACK_TEST_TOKEN")?.into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let session = _client.open_session(&token);

    session
        .api_test(&SlackApiTestRequest::new().with_foo("Test".into()))
        .await?;

    println!("Command event{:#?}", event);
    Ok(SlackCommandEventResponse::new(
        SlackMessageContent::new().with_text("Working on it".into()),
    ))
}

fn test_error_handler(
    err: Box<dyn std::error::Error + Send + Sync>,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> HttpStatusCode {
    println!("{:#?}", err);

    // Defines what we return Slack server
    HttpStatusCode::BAD_REQUEST
}


#[derive(Debug)]
struct UserStateExample(u64);
lazy_static! {
    static ref SLACK_CLIENT: Arc<SlackHyperClient> = {
        // Initialize the client and store it in an Arc
        Arc::new(SlackClient::new(SlackClientHyperConnector::new().unwrap()))
    };
}

async fn events_api() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // let client: Arc<SlackHyperClient> =
    //     Arc::new(SlackClient::new(SlackClientHyperConnector::new()?));
    
    // We clone the client so that we can use it in our async task
    let client = SLACK_CLIENT.clone();

    // We define the address that we want our server to listen on
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));

    // We log the address that we're listening on
    info!("Loading server: {}", addr);

    // We spawn an async task that will run our request consumer
    // The request consumer is a function that will continuously process any incoming requests
    tokio::task::spawn(async move {
        // We call the request_consumer function and await its result
        // If anything goes wrong, we'll get an error here
        let _ = request_consumer().await; 
    });

    async fn request_consumer()
    -> Result<Response<BoxBody<Bytes, Infallible>>, Box<dyn std::error::Error + Send + Sync>> {
        // This is a loop that will run indefinitely
        // Its purpose is to process any incoming requests we have
        loop {
            // First, we need to check if there are any requests waiting
            // We do this by locking the queue and checking its length
            // If there are requests waiting, we pop the last one from the queue
            // If there are no requests waiting, we just skip to the end of the loop
            let request = {
                let mut queue = GLOBAL_QUEUE.lock().unwrap();
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
                request_server(request).await;
            }
            // If there were no requests waiting, we just sleep for a second
            else {
                time::sleep(Duration::from_millis(1)).await;
            }
        }
    }

    /// Handle a single incoming request
    ///
    /// This function is called in an infinite loop in `request_consumer`.
    /// It takes a single JSON value as an argument, which is expected to be
    /// the payload of an incoming request.
    ///
    /// This function will:
    ///
    /// 1. Check if the request is an "event_callback" event.
    /// 2. If so, it will extract the "event" field from the JSON value and
    ///    parse it into a `SlackMessageEvent`.
    /// 3. If the event is a message event, it will check the subtype of the
    ///    event and handle it accordingly.
    /// 4. If the event is not a message event, it will print the event to the
    ///    console.
    async fn request_server(json_value: Value){
        if let Some(event_type) = json_value.get("type").and_then(|v| v.as_str()) {
            match event_type {
                // Handle event_callback events
                "event_callback" => {
                    if let Some(event) = json_value.get("event"){
                        if let Ok(message_event)  = serde_json::from_value::<SlackMessageEvent>(event.clone()){
                            // Handle message events
                            if let Some(subtype) = message_event.subtype{
                                // Handle subtypes of message events
                                match subtype{
                                    // Handle bot messages
                                    SlackMessageEventType::BotMessage => todo!(),
                                    // Handle me messages
                                    SlackMessageEventType::MeMessage => todo!(),
                                    // Handle channel join events
                                    SlackMessageEventType::ChannelJoin => todo!(),
                                    // Handle channel leave events
                                    SlackMessageEventType::ChannelLeave => todo!(),
                                    // Handle bot add events
                                    SlackMessageEventType::BotAdd => todo!(),
                                    // Handle bot remove events
                                    SlackMessageEventType::BotRemove => todo!(),
                                    // Handle channel topic change events
                                    SlackMessageEventType::ChannelTopic => todo!(),
                                    // Handle channel purpose change events
                                    SlackMessageEventType::ChannelPurpose => todo!(),
                                    // Handle channel name change events
                                    SlackMessageEventType::ChannelName => todo!(),
                                    // Handle file share events
                                    SlackMessageEventType::FileShare => todo!(),
                                    // Handle message change events
                                    SlackMessageEventType::MessageChanged => todo!(),
                                    // Handle message delete events
                                    SlackMessageEventType::MessageDeleted => todo!(),
                                    // Handle thread broadcast events
                                    SlackMessageEventType::ThreadBroadcast => todo!(),
                                    // Handle tombstone events
                                    SlackMessageEventType::Tombstone => todo!(),
                                    // Handle joiner notification events
                                    SlackMessageEventType::JoinerNotification => todo!(),
                                    // Handle slackbot response events
                                    SlackMessageEventType::SlackbotResponse => todo!(),
                                    // Handle emoji change events
                                    SlackMessageEventType::EmojiChanged => todo!(),
                                    // Handle huddle room created events
                                    SlackMessageEventType::SlackHuddleRoomCreated => todo!(),
                                    // Handle channel archive events
                                    SlackMessageEventType::ChannelArchive => todo!(),
                                    // Handle channel unarchive events
                                    SlackMessageEventType::ChannelUnarchive => todo!(),
                                    // Handle app conversation leave events
                                    SlackMessageEventType::AppConversationLeave => todo!(),
                                    // Handle bot enable events
                                    SlackMessageEventType::BotEnable => todo!(),
                                    // Handle bot disable events
                                    SlackMessageEventType::BotDisable => todo!(),
                                    // Handle pinned item events
                                    SlackMessageEventType::PinnedItem => todo!(),
                                    // Handle reminder add events
                                    SlackMessageEventType::ReminderAdd => todo!(),
                                    // Handle file comment events
                                    SlackMessageEventType::FileComment => todo!(),
                                    // Handle file created events
                                    SlackMessageEventType::FileCreated => todo!(),
                                    // Handle file changed events
                                    SlackMessageEventType::FileChanged => todo!(),
                                    // Handle file deleted events
                                    SlackMessageEventType::FileDeleted => todo!(),
                                    // Handle file shared events
                                    SlackMessageEventType::FileShared => todo!(),
                                    // Handle file unshared events
                                    SlackMessageEventType::FileUnshared => todo!(),
                                    // Handle file public events
                                    SlackMessageEventType::FilePublic => todo!(),
                                    _=> todo!(),
                                }
                                
                                
                            }
                            else {
                                // Handle regular messages
                                println!("Regular message")
                            }
                            
                        }
                        
                        else if let Some(reaction_type) = event.get("type").and_then(|v| v.as_str()) {
                            match reaction_type {
                                "reaction_added" => {
                                    if let Ok(_event_t) = serde_json::from_value::<SlackReactionAddedEvent>(event.clone()) {
                                        // Handle reaction added events
                                        
                                    } 
                                }
                                "reaction_removed" => {
                                    if let Ok(_event_t) = serde_json::from_value::<SlackReactionRemovedEvent>(event.clone()) {
                                        // Handle reaction removed events
                                    
                                    }
                                }
                                _ => {
                                    println!("Unknown event type: {}", reaction_type);
                                }
                            }
                        }
                        else {
                            println!("{:#?}", event); 
                        }
                    }

                }
                _ => {
                    // Handle other event types if needed
                }
            }
        }
    }

    // ! Where most of our Events will be defined 
    async fn main_event_api(  
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
                        println!("Challenge received, responding with challenge");

                        // We respond with the challenge
                        return Response::builder()
                            .status(200)
                            .header("Content-type", "text/plain")
                            .body(Full::new(challenge.into()).boxed())
                            .map_err(|e| e.into());
                    }
                    else {
                        // If the event is not a challenge, we need to add it to the queue
                        println!("Event Received: {:#?}", json_value);

                        // We add the event to the queue
                        GLOBAL_QUEUE.lock().unwrap().push(json_value); 
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
                    println!("Code received, handling installation request");

                    // We need to handle the installation request
                    let temp_code = SlackOAuthCode::new(code.to_string());
                    let temp_client = SLACK_CLIENT.clone(); 
                    //let temp_client  = SlackClient::new(SlackClientHyperConnector::new()?);

                    let oauth_token_request = SlackOAuthV2AccessTokenRequest::new(
                        config_env_var("SLACK_CLIENT_ID")?.into(),
                        config_env_var("SLACK_CLIENT_SECRET")?.into(),
                        temp_code,
                    );

                    // use the client variable here
                    let _ = temp_client.oauth2_access(&oauth_token_request).await?;
                    
                } else {
                    println!("Unknown GET request");
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

    // Clone the configuration for the push events listener
    let push_events_config = Arc::new(SlackPushEventsListenerConfig::new(
        config_env_var("SLACK_SIGNING_SECRET")?.into(),
    ));

    // Clone the configuration for the interaction events listener
    let interactions_events_config = Arc::new(SlackInteractionEventsListenerConfig::new(
        config_env_var("SLACK_SIGNING_SECRET")?.into(),
    ));

    // Clone the configuration for the command events listener
    let command_events_config = Arc::new(SlackCommandEventsListenerConfig::new(
        config_env_var("SLACK_SIGNING_SECRET")?.into(),
    ));

    // Create a new environment for the listeners
    let listener_environment = Arc::new(
        SlackClientEventsListenerEnvironment::new(client.clone())
            // Set the error handler for the listeners
            .with_error_handler(test_error_handler)
            // Set the initial user state for the listeners
            .with_user_state(UserStateExample(0)),
    );

    // Bind the listener to the specified address
    let listener_tcp = TcpListener::bind(&addr).await?;

    // Log the server address
    info!("Server is listening on {}", &addr);

    // Loop indefinitely to accept incoming connections
    loop {
        // Accept an incoming connection
        let (tcp, _) = listener_tcp.accept().await?;

        // Create a new Tokio Io object from the TCP connection
        let io = TokioIo::new(tcp);

        // Clone the configuration for the push events listener
        let thread_push_events_config = push_events_config.clone();

        // Clone the configuration for the interaction events listener
        let thread_interaction_events_config = interactions_events_config.clone();

        // Clone the configuration for the command events listener
        let thread_command_events_config = command_events_config.clone();

        // Create a new listener object
        let listener = SlackClientEventsHyperListener::new(listener_environment.clone());

        // Define the routes for the listener
        let routes = chain_service_routes_fn(
            // Handle push events
            listener.push_events_service_fn(
                thread_push_events_config,
                test_push_events_function),
            chain_service_routes_fn(
                // Handle interaction events
                listener.interaction_events_service_fn(
                    thread_interaction_events_config,
                    test_interaction_events_function,
                ),
                chain_service_routes_fn(
                    // Handle command events
                    listener.command_events_service_fn(
                        thread_command_events_config,
                        test_command_events_function,
                    ),
                    // Handle other events
                    main_event_api,
                ),
            ),
        );

        // Spawn a new task to handle the incoming connection
        tokio::task::spawn(async move {
            // Serve the connection
            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service_fn(routes))
                .await
  
            {
                // Log any errors that occurred
                println!("Error serving connection: {:?}", err);
                 
            }
        });   
    }
}

pub fn config_env_var(name: &str) -> Result<String, String> {
    // Returns a `String` containing the value of the environment variable with the given `name`.
    // If the environment variable does not exist, returns an error.
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

// Define the global queue as a Mutex wrapped around a vector of QueueItem
lazy_static::lazy_static! {
    static ref GLOBAL_QUEUE: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Loads the environment variables from the `.env` file in the current working directory
    // into the process's environment.
    dotenv().ok();

    // Define the path to the JSON file
    let file_path = "manifest.json";

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

    // Start ngrok in the background
    let _ngrok_process = Command::new("ngrok")
        .arg("http")
        .arg("http://localhost:8080") // Expose port 8080 to the public
        .stdout(Stdio::null()) // Redirect output to null to avoid clutter
        .stderr(Stdio::null()) // Redirect errors to null
        .spawn() // Spawn the process in the background
        .expect("Failed to start ngrok");

    // Send a request to the ngrok API
    let response = reqwest::get("http://127.0.0.1:4040/api/tunnels")
        .await?
        .json::<serde_json::Value>()  // Parse the response as JSON
        .await?;

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
    env::set_var("SLACK_APP_ID", created_app_reponse.app_id.to_string());
    env::set_var("SLACK_CLIENT_ID", created_app_reponse.credentials.client_id.to_string());
    env::set_var("SLACK_CLIENT_SECRET", created_app_reponse.credentials.client_secret.to_string());
    env::set_var("SLACK_VERIFICATION_TOKEN", created_app_reponse.credentials.verification_token.to_string());
    env::set_var("SLACK_SIGNING_SECRET", created_app_reponse.credentials.verification_token.to_string());
    env::set_var("SLACK_OAUTH_REDIRECT_URL", created_app_reponse.oauth_authorize_url.to_string());

    match manifest_struct.clone().oauth_config{
        Some(oauth) => {
            match oauth.clone().scopes
            {
                Some(scopes) => {
                    match scopes.clone().user
                    {
                        Some(user) => {
                            // Set the environment variable `SLACK_USER_SCOPE` to the user scopes
                            env::set_var("SLACK_USER_SCOPE", user
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>()
                            .join(" "));
                        }
                        None => {
                            // Set the environment variable `SLACK_USER_SCOPE` to an empty string
                            env::set_var("SLACK_USER_SCOPE", " ".to_string());
                        }
                    }
                    match scopes.clone().bot
                    {
                        Some(bot) => {
                            // Set the environment variable `SLACK_BOT_SCOPE` to the bot scopes
                            env::set_var("SLACK_BOT_SCOPE", bot
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>()
                            .join(" "));
                        }
                        None => {
                            // Set the environment variable `SLACK_BOT_SCOPE` to an empty string
                            env::set_var("SLACK_BOT_SCOPE", " ".to_string());
                        }
                    }
                }
                None => {
                    
                }
            }
        }
        None => {}
    }
    println!("OAUTH Redirect URL: {:?}", config_env_var("SLACK_OAUTH_REDIRECT_URL")?);
    
    events_api().await?;

    Ok(())
}
