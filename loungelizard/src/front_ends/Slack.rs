
use dioxus:: prelude::*;
// Api mongo structs
use futures::executor::block_on;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::api::mongo_format::mongo_structs::*;
use crate::comp::slack::*;
use slack_morphism::prelude::*;

#[component]
pub fn Slack_fe(

    public_channels:    Signal<Vec<SlackChannelInfo>>,
    private_channels:   Signal<Vec<SlackChannelInfo>>,
    mpim_channels:      Signal<Vec<SlackChannelInfo>>,
    im_channels:        Signal<Vec<SlackChannelInfo>>,
    event_messages:     Signal<Vec<SlackMessageEvent>>,
    history_list:       Signal<Vec<SlackHistoryMessage>>,
    current_channel:    Signal<Option<SlackChannelInfo>>,

) -> Element {

    let user_lock: Signal<Arc<Mutex<User>>> = use_context::<Signal<Arc<Mutex<User>>>>();

    

    use_effect( move || {
        let _ = current_channel();
        event_messages.write().clear();  // Clear old messages
    });

    let mut send_message = use_signal(|| "".to_string());

    let id_lock = user_lock().clone();
    let user_id = {
        block_on(
            async{
                let user = id_lock.lock().await;
                user.slack.user.id.to_string()
            }
        ) 
    };

    let name_fn = ||
        {
            match current_channel() {
                Some(channel) => {
                    match channel.name {
                        Some(name) => {
                            name
                        },
                        None => {
                            "".to_string()
                        }
                    }
                },
                None => {
                    "".to_string()
                }
                
            }
        }; 

    let handle_send_message = move |_| {
        if send_message() != "".to_string()
        {
            block_on(
                async move {
                    let user_lock_c = user_lock().clone();
                    let user = user_lock_c.lock().await;
                    // Create a new Slack client 
                    let client  = 
                    SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));
    
                    // Create a new token from the environment variable `SLACK_CONFIG_TOKEN`
                    let token: SlackApiToken = SlackApiToken::new(user.slack.user.token.clone().into());
    
                    // Create a new session with the client and the token
                    let session = client.open_session(&token);
    
                    match current_channel() {
                        Some(channel) => {
    
                            let message_content = 
                                SlackMessageContent::new()
                                    .with_text(send_message().to_string());
    
                            let post_chat_response = 
                                SlackApiChatPostMessageRequest::new(
                                   channel.id.clone(),
                                   message_content
                                );
                            let _post_message_response=
                                session.chat_post_message(&post_chat_response)
                                .await
                                .unwrap();
    
                            send_message.set("".to_string());
                        },
                        None => {}
                    }
                            
                }
            );
        }
        
    };
    
     rsx!(
        div {
            style: "display: flex; width: 100%;  overflow: hidden; height: 99%; border-radius: 10px; padding: 10px; background-color: rgba(255, 255, 255, 0);",
            div {
                style: "width: 200px; background-color: rgba(44, 47, 51, 0.4); padding: 10px; border-radius: 10px; border-bottom-left-radius: 10px; overflow-y: auto;", // Set fixed width and scrollable if necessary
                // Channels list
                h2 {
                    style: "color: #ADD8E6; margin-bottom: 10px; font-weight: bold;",
                    "Channels"
                }
                ul {
                    style: "list-style-type: none; padding: 0; margin: 0;",
                    for channel in public_channels.iter() {
                        CH_DM_Component{
                            channel_info: channel.clone(),
                            selected_channel: current_channel
                        }
                    }
                }
                hr { style: "border: 1px solid #aaa;" }, // Horizontal line separator
                h2 {
                    style: "color: red; margin-bottom: 10px; margin-top: 10px; font-weight: bold;",
                    "Direct Messages"
                }
                ul {
                    style: "list-style-type: none; padding: 0; margin: 0;",
                    //Replace these list items with your actual channel names
                    for channel in mpim_channels.iter() {
                        CH_DM_Component{
                            channel_info: channel.clone(),
                            selected_channel: current_channel
                            
                        }
                    }
                    // Add more channels as needed
                }
            },
            div{
                style: "display: flex; flex-direction: column; height: 100%; width: 100%; padding : 5px; background-color: rgba(44, 47, 51, 0);",
                div {
                    style: 
                        "display: flex; 
                        flex-direction: column; 
                        height: 100%; 
                        width: 100%; padding: 5px;
                        background-color: rgba(44, 47, 51, 0);
                        color: white;",
                    h1 { 
                        style: "color: #ADD8E6; margin-bottom: 10px;", // Styling for channel name and message list
                        "# {name_fn()}" },
                    ul {
                        style: " flex-grow: 1; overflow-y: auto; border: 1px solid grey; background-color: rgba(255, 255, 255, 0); padding: 10px; width: 100%;",
                        // Render each message inside the scrollable panel
                       
                        for message_h in history_list().iter().rev() {
                            li {
                                style: match &message_h.sender.user {
                                    Some(user) => format!(
                                        "display: flex; flex-direction: column; align-items: {}; margin-bottom: 10px;",
                                        if user.to_string() == user_id { "flex-end" } else { "flex-start" }
                                    ),
                                    None => String::new(), // Return an empty string if there is no user
                                },
                                div {
                                    style: match &message_h.sender.user {
                                        Some(user) => format!(
                                            "padding: 8px; border-radius: 8px; background-color: {}; color: white; max-width: 60%;",
                                            if user.to_string() == user_id {"#6CA6E1"} else {"#8A2BE2"}
                                        ),
                                        None => String::new(), // Return an empty string if there is no user
                                    },
                                    match &message_h.content.text {
                                        Some(text) => format!("{}", text), // Display the message content if available
                                        None => "No content available".to_string(), // Fallback text if there is no content
                                    }
                                }
                                div {
                                    style: "font-size: 0.8em; color: gray; margin-top: 4px;", // Styling for user and timestamp
                                    match &message_h.sender.user {
                                        // TODO Change to usernames 
                                        Some(username) => format!("{}-{}",
                                            if username.to_string() == user_id {"Me".to_string()} else {username.to_string()},
                                            format_timestamp(message_h.origin.ts.to_string()) ), // Display the username if it exists

                                        None => format!("Unknown - {}", format_timestamp(message_h.origin.ts.to_string())), // Fallback text if the username is None
                                    } 
                                } 
                            }
                        
                        }
                        for message in event_messages(){
                            EventMessageComponent{
                                info: message.clone(),
                                id: user_id.clone(),
                            }
                        }
                    },
                    div{
                        style: "display: flex; margin-top: 10px; height: 10%; border: 1px solid white; background-color: #333;",
                        textarea{
                            style: "resize: none; width: 100%; height: 99%; padding: 5px; border-radius: 4px; background-color: #333; color: white; overflow-y: auto; ",
                            value: "{send_message}",
                            placeholder: "Send message...",
                            oninput: move |event| send_message.set(event.value()),
                            
                        }
                        img {
                            src: "assets/send.png",
                            alt: "Send",
                            style: "cursor: pointer; padding: 5px; padding-top: 25px; background-color: #333;;",
                            onclick: handle_send_message,
                        }
                       
                    }
                }
            }

        }
     )
}

pub async fn get_history_list (
    token: String, 
    channel: SlackChannelInfo) -> Vec<SlackHistoryMessage> {

    // Create a new Slack client 
    let client  = 
        SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));

    // Create a new token from the environment variable `SLACK_CONFIG_TOKEN`
    let token: SlackApiToken = SlackApiToken::new(token.into());

    // Create a new session with the client and the token
    let session = client.open_session(&token);

    let get_channel_message_history = 
        SlackApiConversationsHistoryRequest::new()
            .with_channel(channel.id.clone())
            .with_limit(999)
            .with_inclusive(true);

    let get_channel_message_history_response: SlackApiConversationsHistoryResponse = 
        session
            .conversations_history(&get_channel_message_history)
            .await
            .unwrap();

    get_channel_message_history_response.messages

}
fn format_timestamp(timestamp: String) -> String {
    // Parse the timestamp string into a DateTime object
    let parsed_timestamp = DateTime::parse_from_rfc3339(timestamp.as_str()).unwrap_or_else(|_| Utc::now().into());
    
    // Format the date into a readable format, e.g., "Sep 26, 2024 12:45 PM"
    parsed_timestamp.format("%b %d, %Y %I:%M %p").to_string()
}