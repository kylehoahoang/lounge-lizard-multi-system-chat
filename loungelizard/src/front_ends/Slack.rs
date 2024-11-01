
use dioxus:: prelude::*;
// Api mongo structs
use futures::executor::block_on;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::api::mongo_format::mongo_structs::*;
use crate::comp::slack::*;
use slack_morphism::prelude::*;
use std::collections::HashMap;

#[component]
pub fn Slack_fe(

    public_channels:    Signal<Vec<SlackChannelInfo>>,
    private_channels:   Signal<Vec<SlackChannelInfo>>,
    mpim_channels:      Signal<Vec<SlackChannelInfo>>,
    im_channels:        Signal<Vec<SlackChannelInfo>>,
    event_messages:     Signal<HashMap<String, SlackMessageEvent>>,
    event_messages_vec: Signal<Vec<String>>,
    history_list:       Signal<HashMap<String, SlackHistoryMessage>>,
    history_list_vec:   Signal<Vec<String>>,
    current_channel:    Signal<Option<SlackChannelInfo>>,

) -> Element {

    let user_lock: Signal<Arc<Mutex<User>>> = use_context::<Signal<Arc<Mutex<User>>>>();


    let selected_message_id:Signal<Option<String>> = use_signal(||None); 

    use_effect( move || {
        let _ = current_channel();
        event_messages.write().clear();  // Clear old messages
        event_messages_vec.write().clear();
    });

     // Reverse chronological order

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
            match current_channel.read().as_ref() {
                Some(channel) => {
                    match channel.name.clone() {
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
                        style: " flex-grow: 1; overflow-y: auto; overflow-x: hidden; border: 1px solid grey; background-color: rgba(255, 255, 255, 0); padding: 10px; width: 100%;",
                        // Render each message inside the scrollable panel
                       
                        for message_ts in history_list_vec().iter().rev(){
                            if let Some(message_h) = history_list().get(message_ts){
                                
                                CustomMessageComponent{
                                    message: MessageComp{
                                        mess_h: Some(message_h.clone()),
                                        mess_e: None
                                    },
                                    user_id: user_id.clone(),
                                    current_selected_id: selected_message_id.clone()
                                }
                                
                            }
                            
                        }
                        for message_ts in event_messages_vec().iter(){
                            if let Some(message_e) = event_messages().get(message_ts){
                                CustomMessageComponent{
                                    message: MessageComp{
                                        mess_h: None,
                                        mess_e: Some(message_e.clone())
                                    },
                                    user_id: user_id.clone(),
                                    current_selected_id: selected_message_id.clone()
                                }
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
