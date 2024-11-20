
use dioxus:: prelude::*;
// Api mongo structs
use futures::executor::block_on;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::api::{mongo_format::mongo_structs::*, slack::{self, emoji::*}};
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
    user_list:          Signal<HashMap<String, SlackUser>> 

) -> Element {

    // Get the user's Slack token from the context
    let user_lock: Signal<Arc<Mutex<User>>> = use_context::<Signal<Arc<Mutex<User>>>>();

    // The ID of the message that the user has selected
    let selected_message_id:Signal<Option<String>> = use_signal(||None);  // Initialize to None

    // Whether the emoji picker is shown
    let mut emoji_picker = use_signal(|| false);

    // Clear the messages and message IDs when the user changes the channel
    use_effect( move || {
        let _ = current_channel();
        // Clear the old messages and message IDs
        event_messages.write().clear();  
        event_messages_vec.write().clear();
    });

    // Reverse chronological order

    // The text input box for the user to compose a message
    let mut send_message = use_signal(|| "".to_string());
    // The text input box for the user to compose a message to show in the message list
    let mut message_show = use_signal(|| "".to_string());

    // Whether the file upload is enabled
    let mut _file_ex_enable = use_signal(|| false); 
    // The name of the attachment
    let mut attachment_name = use_signal(|| "".to_string());
    // The size of the attachment
    let mut attachment_size: Signal<usize> = use_signal(|| 0);
    // The content of the attachment
    let mut attachment_content = use_signal(|| Vec::<u8>::new());
    // The type of the attachment
    let mut attachment_type = use_signal(|| "".to_string());

    // The ID of the user
    let id_lock = user_lock().clone();
    let user_id = {
        block_on(
            async{
                let user = id_lock.lock().await;
                user.slack.user.id.to_string()
            }
        ) 
    };

    // A function that returns the name of the channel
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
        // Send a message to the Slack channel using the user's token
        // This is a closure that will be called when the user presses the
        // "Send" button in the Slack front-end
        // The closure takes a reference to the user's Slack token
        // and a reference to the channel ID as arguments

        block_on(
            async move {
                // Lock the user's Slack token
                let user_lock_c = user_lock().clone();
                let user = user_lock_c.lock().await;
                // Create a new Slack client
                let client  = 
                SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));

                // Create a new token from the environment variable `SLACK_CONFIG_TOKEN`
                let token: SlackApiToken = SlackApiToken::new(user.slack.user.token.clone().into());
                let channel_id = current_channel(); 

                // Create a new session with the client and the token
                let session = client.open_session(&token);

                match current_channel() {
                    Some(channel) => {

                        // Upload any files that the user has selected
                        let mut files = Vec::<SlackFile>::new(); 
                        if !attachment_name().is_empty() && attachment_size() > 0{
                            // Get the URL to upload the file to
                            let upload_url_request = SlackApiFilesGetUploadUrlExternalRequest::new(
                                attachment_name().to_string(),
                                attachment_size()
                            );
                            let upload_url_response = session.get_upload_url_external(&upload_url_request).await.unwrap();
                            println!("Upload URL Response: {:#?}", upload_url_response);

                            // Upload the file to the URL
                            let file_upload_request = SlackApiFilesUploadViaUrlRequest::new(
                                upload_url_response.upload_url,
                                attachment_content(),
                                attachment_type()
                            );

                            let _ = session.files_upload_via_url(&file_upload_request).await.unwrap();

                            // Complete the upload by sending the file ID and title to Slack
                            let upload_complete_request = SlackApiFilesCompleteUploadExternalRequest::new(
                                vec![SlackApiFilesComplete::new(upload_url_response.file_id).with_title(attachment_name())]
                            ).with_channel_id(channel_id.unwrap().id);

                            let upload_complete_response = session.files_complete_upload_external(&upload_complete_request).await.unwrap();
                            println!("Upload URL Complete Response: {:#?}", upload_complete_response);
                            
                            // Add the uploaded file to the list of files
                            for file in upload_complete_response.files{
                                files.push(file); 
                            }
                            
                        }
                        if !send_message().is_empty()
                        {

                            // Post a message to the Slack channel with the
                            // message content and the uploaded files
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

                            // Clear the message field and the attachment fields
                            send_message.set("".to_string());
                            message_show.set("".to_string());
                            attachment_content.write().clear();
                            attachment_name.set("".to_string());
                            attachment_size.set(0);
                            attachment_type.set("".to_string());
                        }

                    },
                    None => {}
                }
                        
            }
        );
    };
    
     rsx!(
        div {
            style: "
            display: flex; width: 100%; 
            overflow: hidden; height: 99%; 
            border-radius: 10px; padding: 10px;
            background-color: rgba(255, 255, 255, 0);",
            div {
                style: "
                width: 200px; background-color: rgba(44, 47, 51, 0.4); 
                padding: 10px; border-radius: 10px; 
                border-bottom-left-radius: 10px; overflow-y: auto;", // Set fixed width and scrollable if necessary
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
                    style: "
                    color: red; margin-bottom: 10px; 
                    margin-top: 10px; font-weight: bold;",
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
                style: "
                display: flex; flex-direction: column; 
                height: 100%; width: 100%; padding : 5px; 
                background-color: rgba(44, 47, 51, 0);",
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
                        style: " 
                        flex-grow: 1; overflow-y: auto; 
                        overflow-x: hidden; border: 1px solid grey; 
                        background-color: rgba(255, 255, 255, 0); 
                        padding: 10px; width: 100%;",
                        // Render each message inside the scrollable panel
                       
                        for message_ts in history_list_vec().iter().rev(){
                            if let Some(message_h) = history_list().get(message_ts){
                                
                                CustomMessageComponent{
                                    message: MessageComp{
                                        mess_h: Some(message_h.clone()),
                                        mess_e: None
                                    },
                                    user_id: user_id.clone(),
                                    current_selected_id: selected_message_id.clone(),
                                    incoming_user: user_list.clone()
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
                                    current_selected_id: selected_message_id.clone(),
                                    incoming_user: user_list.clone()
                                }
                            }
                        }
                    },
                    div{
                        style: "
                        display: flex; margin-top: 10px; 
                        height: 10%; border: 1px solid white; 
                        background-color: #333;",
                        textarea{
                            style: "
                            resize: none; width: 100%; 
                            height: 99%; padding: 5px; 
                            border-radius: 4px; background-color: #333; 
                            color: white; overflow-y: auto; ",
                            value: "{message_show}",
                            placeholder: "Send message...",
                            oninput: move |event| {
                                send_message.set(event.value());
                                message_show.set(event.value());
                            },
                            
                        },
                        if emoji_picker() {
                            div {
                                style: "position: relative; display: inline-block;",
                                
                                // Emoji picker pane (modal-like)
                                div {
                                    style: "
                                        position: absolute;
                                        top: -165px;
                                        left: -210px;
                                        display: grid;
                                        grid-template-columns: repeat(5, 1fr);
                                        gap: 8px;
                                        padding: 12px;
                                        border: 1px solid #ddd;
                                        background-color: #f5f5f5;
                                        border-radius: 8px;
                                        box-shadow: 0px 4px 12px rgba(0, 0, 0, 0.1);
                                        max-width: calc(100vw - 20px); 
                                        max-height: 150px;   
                                        overflow-y: auto;   
                                        z-index: 100;
                                    ",
                                    for (label, emoji) in EMOJIS.entries(){
                                        div {
                                            style: "
                                                font-size: 1.5em;
                                                padding: 6px;
                                                cursor: pointer;
                                                transition: background-color 0.2s;
                                            ",
                                            onclick: move |_| {
                                                send_message.set(format!("{}:{label}:", send_message()));
                                                message_show.set(format!("{}{}", message_show(), emoji));
                                                
                                            },
                                            // Select emoji on click
                                            "{emoji}"
                                        }
                                    }
                                }
                            }
                        }
                        
                        div{
                            style: "display: flex; flex-direction: row;  ",

                             // File input button
                            
                            button{
                                onclick: move |_| { emoji_picker.set(!emoji_picker());},
                                "😊"
                                
                            },
                            div{
                                style:  "
                                display:flex; flex-direction: column; 
                                justify-content: center; align-items: center;",
                                
                                
                                div {
                                    class: "attachment-container",
                                    // Paperclip icon button
                                    label {   
                                        r#for: "file-input",
                                        class: "attachment-button",
                                        svg {
                                            view_box: "0 0 24 24",
                                            width: "20px",
                                            height: "20px",
                                            fill: "none",
                                            xmlns: "http://www.w3.org/2000/svg",
                                            g {
                                                id: "SVGRepo_bgCarrier",
                                                stroke_width: "0",
                                            }
                                            g {
                                                id: "SVGRepo_tracerCarrier",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                            }
                                            g {
                                                id: "SVGRepo_iconCarrier",
                                                path {
                                                    d: "M19.8278 11.2437L12.7074 18.3641C10.7548 20.3167 7.58896 20.3167 5.63634 18.3641C3.68372 16.4114 3.68372 13.2456 5.63634 11.293L12.4717 4.45763C13.7735 3.15589 15.884 3.15589 17.1858 4.45763C18.4875 5.75938 18.4875 7.86993 17.1858 9.17168L10.3614 15.9961C9.71048 16.647 8.6552 16.647 8.00433 15.9961C7.35345 15.3452 7.35345 14.2899 8.00433 13.6391L14.2258 7.41762",
                                                    stroke: "#f5f5f5",
                                                    stroke_width: "0.8399999999999999",
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                }
                                            }
                                        }
                                    
                                        input {
                                            id: "file-input",
                                            r#type: "file",
                                            accept: "",
                                            multiple: false,
                                            style: "display: none;",
                                            onchange: move |evt| {
                                                async move {
                                                    // If there is a file_engine, read the file and write it to the attachment_content
                                                    if let Some(file_engine) = evt.files() {
                                                        let files = file_engine.files();
                                                        for file_name in &files {
                                                            
                                                            // Clear the attachment_content before writing the file
                                                            attachment_content.write().clear();

                                                            // Clear the attachment_name before writing the file
                                                            attachment_name.set(String::new());

                                                            // Read the file from the file_engine
                                                            if let Some(file) = file_engine.read_file(file_name).await {
                                                                // Write the file to the attachment_content
                                                                attachment_content.write().extend(file);

                                                                // Set the attachment_size to the length of the attachment_content
                                                                attachment_size.set(attachment_content.read().len());

                                                                // Use std::path::Path to extract the file name without the path
                                                                let base_name = std::path::Path::new(file_name).file_name()
                                                                .and_then(|name| name.to_str())
                                                                .unwrap_or("Unknown File");

                                                                // Set the attachment_name to the base_name
                                                                attachment_name.set(base_name.to_string());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                            },
                          
                            button {
                                style: "cursor: pointer; padding: 5px; background-color: #333; font-size: 25px;",
                                onclick: handle_send_message,
                                "➤"
                            }
                        
                        }
                        
                    }
                }
            }

        }
     )
}

/// Get the message history of a given channel
///
/// # Parameters
///
/// * `token`: The Slack bot token
/// * `channel`: The Slack channel info
///
/// # Return
///
/// A vector of `SlackHistoryMessage` containing the message history of the given channel
pub async fn get_history_list(
    token: String,
    channel: SlackChannelInfo,
) -> Vec<SlackHistoryMessage> {
    // Create a new Slack client
    let client = SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));

    // Create a new token from the environment variable `SLACK_CONFIG_TOKEN`
    let token: SlackApiToken = SlackApiToken::new(token.into());

    // Create a new session with the client and the token
    let session = client.open_session(&token);

    // Get the channel message history
    let get_channel_message_history = SlackApiConversationsHistoryRequest::new()
        .with_channel(channel.id.clone())
        .with_limit(999)
        .with_inclusive(true);

    // Get the channel message history response
    let get_channel_message_history_response: SlackApiConversationsHistoryResponse =
        session
            .conversations_history(&get_channel_message_history)
            .await
            .unwrap();

    // Return the channel message history
    get_channel_message_history_response.messages
}

/// Get the user list of a given team
///
/// # Parameters
///
/// * `token`: The Slack bot token
/// * `team_id`: The Slack team ID
///
/// # Return
///
/// A vector of `SlackUser` containing the user list of the given team
pub async fn get_user_list(
    token: String,
    team_id: String,

) -> Vec<SlackUser>
{
    // Create a new Slack client
    let client = SlackClient::new(
        SlackClientHyperConnector::new().expect("failed to create hyper connector")
    );

    // Create a new token from the environment variable `SLACK_CONFIG_TOKEN`
    let token: SlackApiToken = SlackApiToken::new(token.into());

    // Create a new session with the client and the token
    let session = client.open_session(&token);

    // Get the user list request
    let user_request = SlackApiUsersListRequest::new()
        .with_team_id(team_id.into());

    // Get the user list response
    let user_response = session.users_list(&user_request).await;

    // Return the user list or an empty vector if there is an error
    match user_response {
        Ok(response) => response.members,
        Err(_) => Vec::new(),
    }
}

