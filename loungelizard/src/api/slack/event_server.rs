use std::borrow::Borrow;
use dioxus:: prelude::*;
use std::sync::{Arc};
use dioxus::hooks::Coroutine;
use tokio::sync::Mutex;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};
use slack_morphism::prelude::*;
use dioxus_logger::tracing::{info, error, warn, Level};
use std::env;
use mongodb::{sync::Client, bson::doc};
use crate::api::mongo_format::mongo_structs::*;
use futures::executor::block_on;

// Imported internal files
use crate::api::slack::{self, server_utils::*};
use crate::api::slack::server_utils::coroutine_enums::Action;

#[derive(Debug)]
struct UserStateExample(u64);

pub async fn events_api(
    user_lock: Arc<Mutex<User>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    let client: Arc<SlackHyperClient> =
        Arc::new(SlackClient::new(SlackClientHyperConnector::new()?));
    
    // We define the address that we want our server to listen on
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));

    // We log the address that we're listening on
    info!("Loading server: {}", addr);

    // We spawn an async task that will run our request consumer
    // The request consumer is a function that will continuously process any incoming requests

    // Clone the configuration for the push events listener
    let push_events_config = Arc::new(SlackPushEventsListenerConfig::new(
        user_lock.lock().await.slack.verif_token.clone().into(),
        
    ));

    // Clone the configuration for the interaction events listener
    let interactions_events_config = Arc::new(SlackInteractionEventsListenerConfig::new(
        user_lock.lock().await.slack.verif_token.clone().into(),
        
    ));

    // Clone the configuration for the command events listener
    let command_events_config = Arc::new(SlackCommandEventsListenerConfig::new(
        user_lock.lock().await.slack.verif_token.clone().into(),
         
    ));

    // Create a new environment for the listeners
    let listener_environment = Arc::new(
        SlackClientEventsListenerEnvironment::new(client.clone())
            // Set the error handler for the listeners
            .with_error_handler(error_handler::error_handler)
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
                push_events::push_events_function),
            chain_service_routes_fn(
                // Handle interaction events
                listener.interaction_events_service_fn(
                    thread_interaction_events_config,
                    interaction_events::interaction_events_function,
                ),
                chain_service_routes_fn(
                    // Handle command events
                    listener.command_events_service_fn(
                        thread_command_events_config,
                        command_events::command_events_function,
                    ),
                    // Handle other events
                    main_events::main_event_api,
                ),
            ),
        );

        
        // Spawn a new task to handle the incoming connection
        tokio::spawn(async move {
            // Serve the connection
            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service_fn(routes))
                .await
  
            {
                // Log any errors that occurred
                error!("Error serving connection: {:#?}", err);
                 
            }
        });   

       
    }

}