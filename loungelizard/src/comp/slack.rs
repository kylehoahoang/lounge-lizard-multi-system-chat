use dioxus:: prelude::*;

// Api mongo structs
use futures::executor::block_on;
use dioxus_logger::tracing::{info, error, warn};
use std::sync::{Arc};
use tokio::sync::{Mutex, oneshot};
use crate::api::mongo_format::mongo_structs::*;
use slack_morphism::prelude::*;
use chrono::{DateTime, Utc, NaiveDateTime};
// ! Message Component 
// Define the MessageComponent

#[component]
pub fn HistoryMessageComponent(
    info: SlackHistoryMessage,
    id: String,   
) -> Element {
    // Determine the background color and alignment based on the "me" prop
    let (background_color, alignment) =  
        match info.sender.user.clone()
        {
            Some(user) => {
                match user
                {
                    id => ("#6CA6E1", "flex-end"), // Light blue
                    _ => ("#800080", "flex-start") // Purple
                }
            },
            None => ("","")
        };

    let message = 
        use_signal(||
            match info.content.text
            {
                Some(text) => text.clone(),
                None => "".to_string()
            }
        );

    let user = 
        use_signal(||
            match info.sender.username
            {
                Some(user) => user.clone(),
                None => "".to_string()
            }
        );


    rsx! {
            div {
                style: format!("display: flex; flex-direction: column; align-items: {}; margin-bottom: 10px;", alignment),
                div {
                    style: format!("padding: 8px; border-radius: 8px; background-color: {}; color: white; max-width: 60%;", background_color),
                    "{message}" // Display the message content
                }
                div {
                    style: "font-size: 0.8em; color: gray; margin-top: 4px;", // Styling for user and timestamp
                    "{user} - " // Display user and timestamp below the message box
                }
            }
        
    }
}


#[component]
pub fn EventMessageComponent(
    info: SlackMessageEvent,
    id: String,   
) -> Element {
    // Determine the background color and alignment based on the "me" prop
    let (background_color, alignment) =  
        match info.sender.user.clone()
        {
            Some(user) => {
                match user
                {
                    id => ("#6CA6E1", "flex-end"), // Light blue
                    _ => ("#800080", "flex-start") // Purple
                }
            },
            None => ("","")
        };

    let message = 
        use_signal(||
            match info.content
            {
                Some(content) => {
                    match content.text
                        {
                            Some(text) => text.clone(),
                            None => "".to_string()
                        }
                },
                None=> "".to_string()
            }
            
        );

    let user = 
        use_signal(||
            match info.sender.username
            {
                Some(user) => user.clone(),
                None => "Me".to_string()
            }
        );

    let time_stamp =
        use_signal(||
            format_timestamp(&info.origin.ts.to_string())
        );


    rsx! {
        div {
            style: format!("display: flex; flex-direction: column; align-items: {}; margin-bottom: 10px;", alignment),
            div {
                style: format!("padding: 8px; border-radius: 8px; background-color: {}; color: white; max-width: 60%;", background_color),
                "{message}" // Display the message content
            }
            div {
                style: "font-size: 0.8em; color: gray; margin-top: 4px;", // Styling for user and timestamp
                "{user} - {time_stamp}" // Display user and timestamp below the message box
            }
        }
    }
}


#[component]
pub fn CH_DM_Component(
    channel_info: SlackChannelInfo,
    selected_channel: Signal<Option<SlackChannelInfo>>,
) -> Element {
    // Determine the background color and alignment based on the "me" prop
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();

    let background = 
    match selected_channel().clone()
    {
        Some(channel) => {
            if channel.id == channel_info.id {
                "#EAD01C" // Light blue
            } else {
                "white" // Purple
            }
        },
        None => "white"
    };

    let channel_name = 
        use_signal(||
            match channel_info.clone().name
            {
                Some(name) => name.clone(),
                None => "".to_string()
            }
        );

    let handle_click = move |_|{
        selected_channel.set(Some(channel_info.clone())); // Clone the channel_info variable
        
    };

    rsx! {
        div {
            li {
                style: format!("margin-bottom: 10px; cursor: pointer; color:{};", background),
                onclick: handle_click,
                "{channel_name}",
                
            }
        }
    }
}
// 


fn format_timestamp(timestamp: &str) -> String {
    // Parse the timestamp string into a DateTime object
    let parsed_timestamp = DateTime::parse_from_rfc3339(timestamp).unwrap_or_else(|_| Utc::now().into());
    
    // Format the date into a readable format, e.g., "Sep 26, 2024 12:45 PM"
    parsed_timestamp.format("%b %d, %Y %I:%M %p").to_string()
}