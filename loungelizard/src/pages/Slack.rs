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
use tokio::sync::Mutex;

#[derive(Debug)]
struct UserStateExample(u64);

use crate::api::slack::server_utils::coroutine_enums::Action;

#[component]
pub fn Slack() -> Element {
    // ! User Mutex Lock to access the user data
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    let client_lock = use_context::<Signal<Arc<Mutex<Option<Client>>>>>();

    // Lock the user context and clone it for async context
    let user_lock_api = Arc::clone(&user_lock());

    let client_lock_new = Arc::clone(&client_lock());
    let user_lock_new = Arc::clone(&user_lock());

    // ! These signals control the front end of the page 

    // ! ======================== ! //

    struct EmptyStruct {}

    let consumer = use_coroutine::<EmptyStruct,_,_>(|_rx| {
        async move {
            loop{
                
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
        }
    });


    use_effect({
        // Step 2: Start function that will retrieve request and store them in a queue
        tokio::spawn(async move {
            let _ = events_api(user_lock_api).await;
        });
    
        // Return a cleanup function if needed (none in this case)
        || {}
    });

    

    // ! ========================= ! //

    rsx! {
        Slack_fe{}, 
    }

}