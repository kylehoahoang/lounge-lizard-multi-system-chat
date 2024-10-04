use dioxus:: prelude::*;

// Api mongo structs
use futures::executor::block_on;
use dioxus_logger::tracing::{info, error, warn};
use mongodb::{sync::Client, bson::doc};
use crate::api::mongo_format::mongo_structs::*;
use crate::api::slack::event_server::*;
use crate::api::slack::{self, server_utils::*};

use crate::front_ends::Slack::*;
use slack_morphism::prelude::*;
use futures_util::StreamExt;

use serde_json::Value;

use std::sync::{Arc};
use tokio::sync::{Mutex, oneshot};

#[derive(Debug)]
struct UserStateExample(u64);
struct EmptyStruct {}

#[component]
pub fn Slack(current_platform: Signal<String>,) -> Element {
    // ! User Mutex Lock to access the user data
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    let client_lock = use_context::<Signal<Arc<Mutex<Option<Client>>>>>();

    // Lock the user context and clone it for async context
    let user_lock_api = Arc::clone(&user_lock());

    let client_lock_new = Arc::clone(&client_lock());
    let user_lock_new = Arc::clone(&user_lock());

     // Create a oneshot channel to signal the task to stop
     let (stop_tx, stop_rx) = oneshot::channel();

    // ! These signals control the front end of the page 

    // ! ======================== ! //

    let consumer = use_coroutine::<EmptyStruct,_,_>(|_rx| {
        async move {
            loop{
                // Gracefully exit the loop if the platform is not Slack
                if !current_platform().eq("Slack") {
                    // Send the stop signal when needed
                    stop_tx.send(()).unwrap();
                    break;
                }
                    
                let json_response = 
                    main_events::request_consumer(user_lock_new.clone(), client_lock_new.clone()).await;

                // Step 4: Error handeling
                match json_response {
                    Ok(response) => {
                        if response != Value::Null{

                            // ! Essentially all events will be callbacks
                            match response.get("event") {
                                Some(event) => {

                                    match event.get("type") {

                                        Some(event_type) => {

                                            match event_type.as_str() {

                                                // ! Main work area for defining what to do with each event
                                                Some("message") => {
                                                    // Handle message events
                                                    match serde_json::from_value::<SlackMessageEvent>(event.clone())
                                                    {
                                                        Ok(message_event) => {
                                                            // Handle message events
                                                            println!("Message Event: {:#?}", message_event);
                                                        }
                                                        Err(e) => { 
                                                            error!("Error with Message Event: {:?}", e);
                                                        }

                                                    }
                                                }
                                                Some("reaction_added") => {
                                                    // Handle reaction added events
                                                }
                                                Some("reaction_removed") => {
                                                    // Handle reaction added events
                                                }
                                                _ => {
                                                    warn!("Unknown event type: {}", event_type);
                                                }
                                                // ! ========================= ! //
                                            }
                                        } 

                                        None => {
                                            warn!("No event type found in response");
                                        }
                                    }
                                }
                                None => {
                                    warn!("No event found in response");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                        
                    }
                }
            }
            info!("Consumer loop ended");
        }
    });


    use_effect({
        // // Step 2: Start function that will retrieve request and store them in a queue
        // tokio::spawn(async move {
        //     let _ = events_api(user_lock_api).await;
        // });

        // Spawn the task with a signal to stop
        tokio::spawn(async move {
            tokio::select! {
                _ = events_api(user_lock_api) => {},
                _ = stop_rx => {
                    info!("Stopping request endpoint");
                }
            }
        });
    
        // Return a cleanup function if needed (none in this case)
        || {}
    });

    

    // ! ========================= ! //
    // ! This page will function as a backend to interpret events coming in from Slack

    rsx! {
        Slack_fe{}, 
    }

}