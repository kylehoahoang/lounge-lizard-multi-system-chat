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
    let mut user_list           : Signal<HashMap<String, SlackUser>> = use_signal(||HashMap::<String, SlackUser>::new());
    

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
                                                            // The message event has been successfully parsed into a SlackMessageEvent
                                                            // structure.
                                                            info!("Message Event Received");
                                                            
                                                            // Check if the current channel is set and if the message event's channel
                                                            // matches the current channel ID.
                                                            if let Some(current_c) = current_channel() {
                                                                if let Some(id) = message_event.clone().origin.channel {
                                                                    if current_c.id.eq(&id) {
                                                                        info!("Message Pushed");

                                                                        // Check if the message event has a message field with a content field
                                                                        // (i.e., it's not an empty message). If it does, then we can start
                                                                        // processing the edited message.
                                                                        if let Some(edited_message) = message_event.clone().message{
                                                                            let message_id = edited_message.clone().ts.to_string();

                                                                            // Extract the user ID of the person who edited the message.
                                                                            let edit_user = edited_message.clone().sender.user.expect("No user");

                                                                            // Check if the edited message has a content field. If it does, then
                                                                            // we can start processing the edited message.
                                                                            match edited_message.clone().content{
                                                                                Some(edited_content) => {

                                                                                    // Check if the edited message is already in the history list.
                                                                                    // If it is, then we'll update the existing message with the
                                                                                    // edited content.
                                                                                    let mut found = false;
                                                                                    let mut edited_content_clone = edited_content.clone();
                                                                                    
                                                                                    history_list
                                                                                        .write()
                                                                                        .entry(message_id.clone())
                                                                                        .and_modify(|new_message| {
                                                                                            let temp_reactions = new_message.content.reactions.clone();
                                                                                            found = true;
                                                                                            new_message.content = edited_content;
                                                                                            new_message.edited = Some(SlackMessageEdited {user:edit_user.clone(),ts:edited_message.ts.clone()});
                                                                                            new_message.content.reactions = temp_reactions;
                                                                                        });

                                                                                    // If the edited message is not in the history list, then we'll add
                                                                                    // it to the event messages vector and map.
                                                                                    if !found  {
                                                                                        event_messages
                                                                                            .write()
                                                                                            .entry(message_id.clone())
                                                                                            .and_modify(|new_message| {

                                                                                                let temp_reactions = match new_message.content.clone() {
                                                                                                    Some(content) => match content.reactions {
                                                                                                        Some(reactions) => Some(reactions.clone()),
                                                                                                        None => None
                                                                                                    },
                                                                                                    None => None
                                                                                                };

                                                                                                found = true;
                                                                                                edited_content_clone.reactions = temp_reactions.clone();
                                                                                                new_message.content = Some(edited_content_clone);
                                                                                                new_message.message = Some(edited_message.clone()); 
                                                                                                
                                                                                            });
                                                                                    }
                                                                                }
                                                                                ,None => {
                                                                                    
                                                                                }
                                                                            }
                                                                        }
                                                                        else {
                                                                            // If the message event does not have a message field, then we'll add
                                                                            // it to the event messages vector and map.
                                                                            let message_id = message_event.origin.ts.to_string(); // Adjust based on actual ID field
                                                                            event_messages_vec.write().push(message_id.clone());
                                                                            event_messages.write().insert(message_id.clone(), message_event.clone());

                                                                        }
                                                                       
                                                                        
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            // If there's an error parsing the message event, then we'll log the error
                                                            // and continue to the next message event.
                                                            error!("Error with Message Event: {:?}", e);
                                                        }

                                                    }
                                                }
                                                Some("reaction_added") => {
                                                    // Handle reaction added events
                                                    match serde_json::from_value::<SlackReactionAddedEvent>(event.clone()) {
                                                        Ok(reaction_item) => {
                                                            // Extract user ID and reaction name from the reaction item
                                                            let user_id = reaction_item.user.clone();
                                                            let reaction_name = reaction_item.reaction;

                                                            // Match the item type in the reaction (e.g., Message or File)
                                                            match reaction_item.item {
                                                                SlackReactionsItem::Message(message) => {
                                                                    // Log the reception of a reaction added event
                                                                    info!("->Reaction Added Event Received");

                                                                    // Check if there is a current channel
                                                                    if let Some(current_c) = current_channel() {
                                                                        // Check if the event belongs to the current channel
                                                                        if let Some(id) = message.clone().origin.channel {
                                                                            if current_c.id.eq(&id) {
                                                                                // Flag to check if the reaction is found
                                                                                let mut found = false;

                                                                                // Attempt to update the history list with the new reaction
                                                                                history_list
                                                                                    .write()
                                                                                    .entry(message.clone().origin.ts.to_string())
                                                                                    .and_modify(|new_message| {
                                                                                        // Check if the message already has reactions
                                                                                        if let Some(new_reactions) = new_message.content.reactions.as_mut() {
                                                                                            // Iterate through current reactions to find a match
                                                                                            info!("Found in reactions");
                                                                                            for reaction in new_reactions.iter_mut() {
                                                                                                if reaction.name == reaction_name {
                                                                                                    info!("Reaction found");
                                                                                                    reaction.count += 1;
                                                                                                    reaction.users.push(user_id.clone());
                                                                                                    found = true;
                                                                                                }
                                                                                            }
                                                                                            // If no matching reaction is found, add a new one
                                                                                            if !found {
                                                                                                info!("Reaction added");
                                                                                                new_reactions.push(SlackReaction {
                                                                                                    name: reaction_name.clone(),
                                                                                                    count: 1,
                                                                                                    users: vec![user_id.clone()],
                                                                                                });
                                                                                            }
                                                                                        } else {
                                                                                            // Initialize reactions vector if none exists
                                                                                            info!{"No current vector for reactions, creating one"};
                                                                                            new_message.content.reactions = Some(vec![SlackReaction {
                                                                                                name: reaction_name.clone(),
                                                                                                count: 1,
                                                                                                users: vec![user_id.clone()],
                                                                                            }]);
                                                                                        }
                                                                                    });

                                                                                // Attempt to update the event messages if not found in history
                                                                                if !found {
                                                                                    event_messages
                                                                                        .write()
                                                                                        .entry(message.clone().origin.ts.to_string())
                                                                                        .and_modify(|new_message| {
                                                                                            // Check if message content exists
                                                                                            if let Some(content) = new_message.content.as_mut() {
                                                                                                // Check if reactions exist
                                                                                                if let Some(reactions) = content.reactions.as_mut() {
                                                                                                    // Iterate through current reactions to find a match
                                                                                                    for reaction in reactions.iter_mut() {
                                                                                                        if reaction.name == reaction_name {
                                                                                                            info!("Reaction found");
                                                                                                            reaction.count += 1;
                                                                                                            reaction.users.push(user_id.clone());
                                                                                                            found = true;
                                                                                                        }
                                                                                                    }
                                                                                                    // If no matching reaction is found, add a new one
                                                                                                    if !found {
                                                                                                        info!("Reaction added");
                                                                                                        reactions.push(SlackReaction {
                                                                                                            name: reaction_name,
                                                                                                            count: 1,
                                                                                                            users: vec![user_id.clone()],
                                                                                                        });
                                                                                                    }
                                                                                                    // Remove reactions with a count of zero
                                                                                                    reactions.retain(|u| u.count != 0);
                                                                                                } else {
                                                                                                    // Initialize reactions vector if none exists
                                                                                                    info!{"No current vector for reactions, creating one"};
                                                                                                    content.reactions = Some(vec![SlackReaction {
                                                                                                        name: reaction_name.clone(),
                                                                                                        count: 1,
                                                                                                        users: vec![user_id.clone()],
                                                                                                    }]);
                                                                                                }
                                                                                            }
                                                                                        });
                                                                                }
                                                                                
                                                                            }
                                                                        }
                                                                    }
                                                                },
                                                                SlackReactionsItem::File(file) => {
                                                                    // Handle file-based reactions if needed
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            // Log error if deserialization fails
                                                            error!("Error with Message Event: {:?}", e);
                                                        }
                                                    }
                                                }
                                                Some("reaction_removed") => {
                                                    // Handle reaction added events
                                                    // This event is triggered when a user removes a reaction from a message
                                                    // It contains the user ID of the user who removed the reaction and the name of the reaction
                                                    match serde_json::from_value::<SlackReactionRemovedEvent>(event.clone())
                                                    {
                                                        Ok(reaction_item) => {

                                                            // Get the user ID of the user who removed the reaction
                                                            let user_id = reaction_item.user.clone();

                                                            // Get the name of the reaction that was removed
                                                            let reaction_name = reaction_item.reaction;

                                                            // Get the item to which the reaction was added
                                                            match reaction_item.item{
                                                                SlackReactionsItem::Message(message) => {
                                                                    // Handle message events
                                                                    info!("->Reaction Removed Event Received");

                                                                    // Check if the current channel is set
                                                                    if let Some(current_c) = current_channel() {

                                                                        // Check if the message is in the current channel
                                                                        if let Some(id) = message.clone().origin.channel{
                                                                            if current_c.id.eq(&id) {

                                                                                // If the message is in the current channel, update the reaction count in the history list
                                                                                info!("Reaction Item Found In messages");

                                                                                let mut found = false;

                                                                                // Update the reaction count in the history list
                                                                                history_list
                                                                                    .write()
                                                                                    .entry(message.clone().origin.ts.to_string())
                                                                                    .and_modify(|new_message| {

                                                                                        // Check if the message has reactions
                                                                                        if let Some(new_reactions) = new_message.content.reactions.as_mut() {

                                                                                            // Now you can modify the reactions vector
                                                                                            for reaction in new_reactions.iter_mut() {
                                                                                                // Update the reaction count and users if the reaction matches
                                                                                                if reaction.name == reaction_name {
                                                                                                    reaction.count -= 1;
                                                                                                    reaction.users.retain(|u| u != &user_id);
                                                                                                    found = true;
                                                                                                }
                                                                                            }

                                                                                            // Remove reactions with a count of 0
                                                                                            new_reactions.retain(|u| u.count != 0);  
                                                                                        }
                                                                                    });

                                                                                // If the reaction was not found in the history list, update the event messages list
                                                                                if !found {
                                                                                    event_messages
                                                                                        .write()
                                                                                        .entry(message.clone().origin.ts.to_string())
                                                                                        .and_modify(|new_message| {

                                                                                            // Check if the message has content
                                                                                            if let Some(content) = new_message.content.as_mut() {

                                                                                                // Check if the content has reactions
                                                                                                if let Some(reactions) = content.reactions.as_mut() {

                                                                                                    // Now you can access the reactions vector
                                                                                                    for reaction in reactions.iter_mut() {
                                                                                                        // Update the reaction count and users if the reaction matches
                                                                                                        if reaction.name == reaction_name {
                                                                                                            reaction.count -= 1;
                                                                                                            reaction.users.retain(|u| u != &user_id);
                                                                                                        }
                                                                                                    }

                                                                                                    // Remove reactions with a count of 0
                                                                                                    reactions.retain(|u| u.count != 0); 
                                                                                                }
                                                                                            }
                                                                                        });
                                                                                }
                                                                                
                                                                                
                                                                                
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
                // Lock the user lock install to retrieve the user data
                let user = user_lock_install.lock().await;

                // Check if the user has installed the Slack app
                if user.slack.app_id != "" {

                    // Create a new Slack client
                    let client  = 
                        SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));

                    // Create a new token from the environment variable `SLACK_CONFIG_TOKEN`
                    let token: SlackApiToken = SlackApiToken::new(user.slack.user.token.clone().into());

                    // Create a new session with the client and the token
                    let session = client.open_session(&token);

                    // Create a request to get the list of channels
                    let get_channel_request: SlackApiConversationsListRequest = 
                        SlackApiConversationsListRequest::new()
                            .with_exclude_archived(true)
                            .with_types(vec![
                                
                                SlackConversationType::Public,
                                SlackConversationType::Private,
                                SlackConversationType::Mpim,
                                SlackConversationType::Im,
                        ]);

                    // Send the request to get the list of channels
                    let get_channel_response = 
                        session
                            .conversations_list(&get_channel_request)
                            .await
                            .unwrap();

                    // Parse the response to get the list of channels
                    match get_channel_response.channels {
                        channel_list => {

                            // Iterate over the channels and separate them by type
                            for channel in channel_list {
                                match channel.flags.is_channel {
                                    Some(true) => {
                                        // Add the channel to the public channels list
                                        public_channels.push(channel.clone());
                                    },
                                    Some(false) => {
                                        match channel.flags.is_private {
                                            Some(true) => {
                                                // Add the channel to the private channels list
                                                private_channels.push(channel.clone());
                                            },
                                            Some(false) => {
                                                match channel.flags.is_mpim {
                                                    Some(true) => {
                                                        // Add the channel to the mpim channels list
                                                        mpim_channels.push(channel.clone());
                                                    },
                                                    Some(false) => {
                                                        // Add the channel to the im channels list
                                                        im_channels.push(channel.clone());
                                                    },
                                                    None => {} // Do nothing if the channel is not an MPIM
                                                }
                                            },
                                            None => {} // Do nothing if the channel is not private
                                        }
                                    },
                                    None => {} // Do nothing if the channel is not a channel
                                }
                            }

                            // Set the current channel to the first channel of any of the lists
                            current_channel.set(
                                // Get the first channel of any of the lists
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
        
                            // Set installed to true
                            installed.set(true);
                        }
                    };

                }
            }
        );
       
        
    };
    
    
    let lock_temp = Arc::clone(&user_lock());
    use_effect(move ||{
         // Clear old messages
            block_on(
                async{
                    info!("Clearing old messages");
                    // Clear the history list and its vector
                    history_list.write().clear();
                    history_list_vec.write().clear();

                    // If there is a current channel, update the history messages
                    if let Some(chan) = current_channel() {
                        info!("Channel found, updating history messages");
                        let user = lock_temp.lock().await;
                        let token = user.clone().slack.user.token.clone();
                        let team_id = user.clone().slack.team.id.clone();

                        // Update the user list
                        info!("Updating user list");
                        for user in get_user_list(token.clone(), team_id.clone()).await.iter() {
                            let user_id = user.clone().id.to_string();
                            info!("Adding user: {:?}", user_id);
                            user_list.write().insert(user_id.clone(), user.clone());
                        }

                        // Get the history messages for the current channel
                        info!("Getting history messages for channel: {:?}", chan.id);
                        for history_message in get_history_list(token, chan.clone()).await {
                            let message_id = history_message.origin.ts.to_string(); // Adjust based on actual ID field
                            info!("Found history message: {:?}", message_id);
                            let mut moded_hist_mess = history_message;
                            // ! Include Channel ID and type
                            info!("Adding channel ID and type to history message");
                            moded_hist_mess.origin.channel = Some(
                                chan.id.clone()
                            );
                            history_list.write().insert(message_id.clone(), moded_hist_mess.clone());
                            history_list_vec.write().push(message_id.clone());
                            info!("Pushed history message: {:?}", message_id);

                        } 
                        

                    }
                    
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
                user_list:          user_list.clone()
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