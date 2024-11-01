use dioxus:: prelude::*;

// Api mongo structs
use futures::executor::block_on;
use dioxus_logger::tracing::{info, error, warn};
use mongodb::{sync::Client, bson::doc};
use crate::api::mongo_format::mongo_structs::*;
use crate::api::slack::event_server::*;
use crate::api::slack::server_utils::*;
use crate::front_ends::Slack::*;
use slack_morphism::prelude::*;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use std::collections::HashMap;
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

    let mut public_channels     : Signal<Vec<SlackChannelInfo>> = use_signal(||Vec::new()); 
    let mut private_channels    : Signal<Vec<SlackChannelInfo>> = use_signal(||Vec::new());
    let mut mpim_channels       : Signal<Vec<SlackChannelInfo>> = use_signal(||Vec::new());  
    let mut im_channels         : Signal<Vec<SlackChannelInfo>> = use_signal(||Vec::new());
    let mut event_messages      : Signal<HashMap<String, SlackMessageEvent>> = use_signal(||HashMap::<String, SlackMessageEvent>::new());
    let mut event_messages_vec  : Signal<Vec<String>>   = use_signal(|| Vec::<String>::new());
    let mut history_list        : Signal<HashMap<String, SlackHistoryMessage>> = use_signal(||HashMap::<String, SlackHistoryMessage>::new());
    let mut history_list_vec    : Signal<Vec<String>> = use_signal(|| Vec::<String>::new());
    let mut current_channel     : Signal<Option<SlackChannelInfo>> = use_signal(||None);
    

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

    // This is the main consumer loop for the Slack front end. It requests events every 5 seconds and
    // handles the events by pushing them to the correct vector based on the event type.
    //
    // # Important
    // You should not call this function directly. Instead, use the `use_coroutine` macro to
    // spawn this function in a separate task.
    let _consumer = use_coroutine::<EmptyStruct,_,_>(|_rx| {
        async move {
            // The main consumer loop. This will request events every 5 seconds and handle the
            // events by pushing them to the correct vector based on the event type.
            info!("Consumer loop started");
            loop {
                // Gracefully exit the loop if the platform is not Slack
                if !current_platform().eq("Slack") {
                    // Send the stop signal when needed
                    stop_tx.send(()).unwrap();
                    info!("Consumer loop stopped");
                    break;
                }
                    
                let json_response =
                    main_events::request_consumer(user_lock_new.clone(), client_lock_new.clone()).await;

                // Step 4: Error handeling
                match json_response {
                    Ok(response) => {
                        if response != Value::Null {

                            // ! Essentially all events will be callbacks
                            match response.get("event") {
                                Some(event) => {

                                    match event.get("type") {

                                        Some(event_type) => {

                                            match event_type.as_str() {

                                                // ! Main work area for defining what to do with each event
                                                Some("message") => {
                                                    // Handle message events
                                                    info!("Handling message event");
                                                    match serde_json::from_value::<SlackMessageEvent>(event.clone())
                                                    {
                                                        Ok(message_event) => {
                                                            // Handle message events
                                                            info!("Message Event Received");
                                                            if let Some(current_c) = current_channel() {
                                                                if let Some(id) = message_event.clone().origin.channel {
                                                                    if current_c.id.eq(&id) {
                                                                        info!("Message Pushed");
                                                                        let message_id = message_event.origin.ts.to_string(); // Adjust based on actual ID field
                                                                        event_messages_vec.write().push(message_id.clone());
                                                                        event_messages.write().insert(message_id.clone(), message_event.clone());
                                                                        
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
                                                    match serde_json::from_value::<SlackReactionAddedEvent>(event.clone())
                                                    {
                                                        Ok(reaction_item) => {

                                                            let user_id = reaction_item.user.clone();
                                                            let reaction_name = reaction_item.reaction;
                                                            //let item_user = reaction_item.item_user.clone();

                                                            match reaction_item.item{
                                                                SlackReactionsItem::Message(message) => {
                                                                                                                                // Handle message events
                                                                    info!("->Reaction Added Event Received");
                                                                    if let Some(current_c) = current_channel() {
                                                                        
                                                                        if let Some(id) = message.clone().origin.channel{
                                                                            if current_c.id.eq(&id) {
                                                                                
                                                                                let mut found = false;
                                                                                
                                                                                history_list
                                                                                    .write()
                                                                                    .entry(message.clone().origin.ts.to_string())
                                                                                    .and_modify(|new_message| {
                                                                                        if let Some(new_reactions) = new_message.content.reactions.as_mut() {
                                                                                            // Now you can modify the reactions vector
                                                                                            info!("Found in reactions");
                                                                                            for reaction in new_reactions.iter_mut() {
                                                                                                if reaction.name == reaction_name {
                                                                                                    info!("Reaction found");
                                                                                                    reaction.count += 1;
                                                                                                    reaction.users.push(user_id.clone());
                                                                                                    found = true;
                                                                                                }
                                                                                            }
                                                                                            if !found {
                                                                                                info!("Reaction added");
                                                                                                new_reactions.push(SlackReaction{
                                                                                                    name: reaction_name.clone(),
                                                                                                    count: 1,
                                                                                                    users: vec![user_id.clone()],
                                                                                                });
                                                                                            }
                                                                                        }
                                                                                        else {
                                                                                           info!{"No current vector for reactions, creating one"};
                                                                                           new_message.content.reactions = Some(vec![SlackReaction{
                                                                                               name: reaction_name.clone(),
                                                                                               count: 1,
                                                                                               users: vec![user_id.clone()],
                                                                                           }]);
                                                                                        }
                                                                                    });

                                                                                if !found  {
                                                                                    event_messages
                                                                                        .write()
                                                                                        .entry(message.clone().origin.ts.to_string())
                                                                                        .and_modify(|new_message| {

                                                                                            if let Some(content) = new_message.content.as_mut() {
                                                                                                if let Some(reactions) = content.reactions.as_mut() {
                                                                                                    // Now you can access the reactions vector
                                                                                                    for reaction in reactions.iter_mut() {
                                                                                                        if reaction.name == reaction_name {
                                                                                                            info!("Reaction found");
                                                                                                            reaction.count += 1;
                                                                                                            reaction.users.push(user_id.clone());
                                                                                                            found = true;
                                                                                                        }
                                                                                                    }
                                                                                                    if !found {
                                                                                                        info!("Reaction added");
                                                                                                        reactions.push(SlackReaction{
                                                                                                            name: reaction_name,
                                                                                                            count: 1,
                                                                                                            users: vec![user_id.clone()],
                                                                                                        });
                                                                                                    }
                                                                                                    reactions.retain(|u| u.count != 0); 
                                                                                                }
                                                                                                else {
                                                                                                    info!{"No current vector for reactions, creating one"};
                                                                                                    content.reactions = Some(vec![SlackReaction{
                                                                                                        name: reaction_name.clone(),
                                                                                                        count: 1,
                                                                                                        users: vec![user_id.clone()],
                                                                                                    }]);
                                                                                                }

                                                                                            }
                                                                                        
                                                                                        });
                                                                                }
                                                                                // Todo Implement the other list 
                                                                                
                                                                            }
                                                                        }
                                                                    }

                                                                },
                                                                SlackReactionsItem::File(file)=>{

                                                                }
                                                            }

                                                        }
                                                        Err(e) => {
                                                            error!("Error with Message Event: {:?}", e);
                                                        }

                                                    }
                                                    
                                                }
                                                Some("reaction_removed") => {
                                                    // Handle reaction added events
                                                    match serde_json::from_value::<SlackReactionRemovedEvent>(event.clone())
                                                    {
                                                        Ok(reaction_item) => {

                                                            let user_id = reaction_item.user.clone();
                                                            let reaction_name = reaction_item.reaction;
                                                            //let item_user = reaction_item.item_user.clone();

                                                            match reaction_item.item{
                                                                SlackReactionsItem::Message(message) => {
                                                                                                                                // Handle message events
                                                                    info!("->Reaction Removed Event Received");
                                                                    if let Some(current_c) = current_channel() {
                                                                        
                                                                        if let Some(id) = message.clone().origin.channel{
                                                                            if current_c.id.eq(&id) {
                                                                                info!("Reaction Item Found In messages");

                                                                                let mut found = false;

                                                                                history_list
                                                                                    .write()
                                                                                    .entry(message.clone().origin.ts.to_string())
                                                                                    .and_modify(|new_message| {
                                                                                        if let Some(new_reactions) = new_message.content.reactions.as_mut() {
                                                                                            // Now you can modify the reactions vector
                                                                                            for reaction in new_reactions.iter_mut() {
                                                                                                if reaction.name == reaction_name {
                                                                                                    reaction.count -= 1;
                                                                                                    reaction.users.retain(|u| u != &user_id);
                                                                                                    found = true;
                                                                                                }
                                                                                            }
                                                                                            new_reactions.retain(|u| u.count != 0);  
                                                                                        }
                                                                                    });

                                                                                if !found {
                                                                                    event_messages
                                                                                        .write()
                                                                                        .entry(message.clone().origin.ts.to_string())
                                                                                        .and_modify(|new_message| {

                                                                                            if let Some(content) = new_message.content.as_mut() {
                                                                                                if let Some(reactions) = content.reactions.as_mut() {
                                                                                                    // Now you can access the reactions vector
                                                                                                    for reaction in reactions.iter_mut() {
                                                                                                        if reaction.name == reaction_name {
                                                                                                            reaction.count -= 1;
                                                                                                            reaction.users.retain(|u| u != &user_id);
                                                                                                        }
                                                                                                    }
                                                                                                    reactions.retain(|u| u.count != 0); 
                                                                                                }
                                                                                            }
                                                                                        });
                                                                                }
                                                                                
                                                                                // Todo Implement the other list 
                                                                                
                                                                            }
                                                                        }
                                                                    }

                                                                },
                                                                SlackReactionsItem::File(file)=>{

                                                                }
                                                            }

                                                        }
                                                        Err(e) => {
                                                            error!("Error with Message Event: {:?}", e);
                                                        }

                                                    }
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
                    history_list_vec.write().clear();
                    if let Some(chan) = current_channel() {
                        let token = lock_temp.lock().await.slack.user.token.clone();
                        for history_message in get_history_list(token, chan.clone()).await {
                            let message_id = history_message.origin.ts.to_string(); // Adjust based on actual ID field
                            let mut moded_hist_mess = history_message;
                            // ! Include Channel ID and type
                            moded_hist_mess.origin.channel = Some(
                                chan.id.clone()
                            );
                            history_list.write().insert(message_id.clone(), moded_hist_mess.clone());
                            history_list_vec.write().push(message_id.clone());
                            //println!("Pushed history message: {:?}", message_id);
                            
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
                event_messages_vec: event_messages_vec.clone(),
                history_list:       history_list.clone(),
                history_list_vec:   history_list_vec.clone(),
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

// ! ========================= ! //