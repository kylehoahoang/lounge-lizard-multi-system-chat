use dioxus:: prelude::*;

// Api mongo structs
use futures::executor::block_on;
use dioxus_logger::tracing::{info, error, warn};
use mongodb::{sync::Client, bson::doc};
use tokio::runtime::TryCurrentError;
use crate::api::mongo_format::mongo_structs::*;
use crate::api::slack::event_server::*;
use crate::api::slack::{self, server_utils::*};

use crate::comp::slack::*;

use crate::front_ends::Slack::*;
use slack_morphism::prelude::*;
use futures_util::StreamExt;

use serde_json::Value;

use std::result;
use std::sync::{Arc};
use tokio::sync::{Mutex, oneshot};
use slack_morphism::prelude::*;

use tokio::time::{sleep, Duration};

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

    let mut public_channels : Signal<Vec<SlackChannelInfo>> = use_signal(||Vec::new()); 
    let mut private_channels: Signal<Vec<SlackChannelInfo>> = use_signal(||Vec::new());
    let mut mpim_channels   : Signal<Vec<SlackChannelInfo>> = use_signal(||Vec::new());  
    let mut im_channels     : Signal<Vec<SlackChannelInfo>> = use_signal(||Vec::new());
    let mut event_messages  : Signal<Vec<SlackMessageEvent>>   = use_signal(|| Vec::<SlackMessageEvent>::new());
    let mut history_list    : Signal<Vec<SlackHistoryMessage>> = use_signal(|| Vec::<SlackHistoryMessage>::new());
    let mut current_channel : Signal<Option<SlackChannelInfo>> = use_signal(||None);


     // Create a oneshot channel to signal the task to stop
    let (stop_tx, stop_rx) = oneshot::channel();

    // ! Spawn events api
    use_effect({
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

    // ! Fill all necessary components for the page vectors 
    // Messages vector

    let consumer = use_coroutine::<EmptyStruct,_,_>(|_rx| {
        async move {
            info!("Consumer loop started");
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
                                                            info!("Message Event Received");
                                                            if let Some(current_c) = current_channel() {
                                                                if let Some(id) = message_event.clone().origin.channel {
                                                                    if current_c.id.eq(&id) {
                                                                        info!("Message Pushed");
                                                                        event_messages.write().push(message_event);
                                                                        
                                                                    }
                                                                }
                                                            }
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

    let user_lock_install = Arc::clone(&user_lock());
    let user_lock_first_try = Arc::clone(&user_lock());

    let mut installed = use_signal(|| false);

    let go_to = use_signal(|| 
        
        block_on(
            async {
                let  user = user_lock_first_try.lock().await;
                if user.slack.app_id != "" {
                    true
                } else {
                    false
                }
        
    }));

    let handle_enter_click = move |_| {

        block_on(
            async {
                let user = user_lock_install.lock().await;

                if user.slack.app_id != "" {
                    // Create a new Slack client 
                    let client  = 
                        SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));

                    // Create a new token from the environment variable `SLACK_CONFIG_TOKEN`
                    let token: SlackApiToken = SlackApiToken::new(user.slack.user.token.clone().into());

                    // Create a new session with the client and the token
                    let session = client.open_session(&token);

                    let get_channel_request: SlackApiConversationsListRequest = 
                        SlackApiConversationsListRequest::new()
                            .with_exclude_archived(true)
                            .with_types(vec![
                                //TODO Slowly Start Implementing Different Channel Types
                                SlackConversationType::Public,
                                SlackConversationType::Private,
                                SlackConversationType::Mpim,
                                SlackConversationType::Im,
                        ]);

                    let get_channel_response = 
                        session
                            .conversations_list(&get_channel_request)
                            .await
                            .unwrap();

                    match get_channel_response.channels {
                        channel_list => {
                            for channel in channel_list {
                                match channel.flags.is_channel {
                                    Some(true) => public_channels.push(channel.clone()),
                                    Some(false) => {
                                        match channel.flags.is_private {
                                            Some(true) => private_channels.push(channel.clone()),
                                            Some(false) => {
                                                match channel.flags.is_mpim {
                                                    Some(true) => mpim_channels.push(channel.clone()),
                                                    Some(false) => im_channels.push(channel.clone()),
                                                    None => {}
                                                }
                                            },
                                            None => {}
                                        }
                                    },
                                    None => {}
                                }
                            }
                        },
                        _ => {}
                    };

                    current_channel.set(
                                // Get the first channel of any of the 
                                match public_channels.first() {
                                    Some(channel) => Some(channel.clone()),
                                    None => {
                                        match private_channels.first() {
                                            Some(channel) => Some(channel.clone()),
                                            None => {
                                                match mpim_channels.first() {
                                                    Some(channel) => Some(channel.clone()),
                                                    None => {
                                                        match im_channels.first() {
                                                            Some(channel) => Some(channel.clone()),
                                                            None => None
                                                        }
                                                    }
                                                }
                                            }
                                            
                                        }
                                    }
                                        // provide default values for the fields of SlackChannelIn,
                                }
                            );
        
                    installed.set(true);
                }
            }
        );
       
        
    };
    
    let lock_temp = Arc::clone(&user_lock());
    use_effect(move ||{
         // Clear old messages
            block_on(
                async{
                    history_list.write().clear();
                    if let Some(chan) = current_channel() {
                        let token = lock_temp.lock().await.slack.user.token.clone();
                        for history_message in get_history_list(token, chan).await {
                            println!("History List: {:#?}", history_message);
                            history_list.write().push(history_message);
                            
                        }
                        
                    }
                    else {}
                }
            );
            
    });

    

    // ! ========================= ! //
    // ! This page will function as a backend to interpret events coming in from Slack

    rsx! {

        if installed() {
            Slack_fe{
                public_channels:    public_channels.clone(),
                private_channels:   private_channels.clone(),
                mpim_channels:      mpim_channels.clone(),
                im_channels:        im_channels.clone(),
                event_messages:     event_messages.clone(),
                history_list:       history_list.clone(),
                current_channel:    current_channel.clone(),
            }
        }
        else {
            

            if go_to() {
                h2 { 
                    class: "welcome-message", 
                    "Welcome Back!"
                }
                button { 
                    class: "login-button",
                    onclick: handle_enter_click,
                    "Go Ahead",
                }
            }
            else {
                h2 { 
                    class: "welcome-message", 
                    "Remeber to install the app on Slack first!"
                }
                button { 
                    class: "login-button",
                    onclick: handle_enter_click,
                    "Click Once Installed",
                }
            }
            
            

        }
        
    }

}

