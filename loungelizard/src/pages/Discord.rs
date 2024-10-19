use dioxus::prelude::*;
use serde_json::Value;
use tracing::info;
use futures::executor::block_on;
use chrono::{DateTime, Utc};
use crate::api::discord::discord_api::*;

// Api mongo structs
use crate::api::mongo_format::mongo_structs::*;
use std::sync::Arc;
use tokio::sync::Mutex;
#[component]
pub fn Discord(show_discord_server_pane: Signal<bool>, discord_guilds: Signal<Value>) -> Element {
   // ! User Mutex Lock to access the user data
   let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
   
   let user_guilds = Arc::clone(&user_lock());
   
   // ! ========================= ! //

   block_on(async move {
    let discord_token = user_guilds.lock().await.discord.token.clone();

    match get_guilds(discord_token).await {
        Ok(discord_guilds_response) => {
            discord_guilds.set(discord_guilds_response); // Call the success handler
            info!("discord_guilds get successful");
        }
        Err(e) => {
            //login_error.set(Some(e.to_string()));
            info!("discord_guilds get failed: {}", e);
            }
        }
    });
   
   
    rsx! { 
        // Bottom pane for servers
        DiscordBottomPane { 
            show_discord_server_pane: show_discord_server_pane.clone(),
            discord_guilds: discord_guilds.clone(),
            user: user_lock
        }, 
    }
}

#[component]
fn DiscordBottomPane(show_discord_server_pane: Signal<bool>, discord_guilds: Signal<Value>, user: Signal<Arc<Mutex<User>>>) -> Element {
    let discord_guilds_array = discord_guilds().as_array().unwrap_or(&vec![]).clone();
    let mut channels = use_signal(|| None::<Value>);
    let mut fetch_error = use_signal(|| None::<String>);
    let mut show_channel_pane = use_signal(|| false);

    // Fetch the channels for the selected guild
    let handle_get_channels = move |guild_id: String, user_lock_api: Arc<Mutex<User>>| {
        block_on(async move {
            let user_lock_api = user_lock_api.lock().await;
            let discord_token = user_lock_api.discord.token.clone();
            match get_guild_channels(discord_token, guild_id).await {
                Ok(channels_data) => {
                    channels.set(Some(channels_data));
                    show_channel_pane.set(true);
                }
                Err(e) => {
                    fetch_error.set(Some(e.to_string()));
                }
            }
        });
    };

    rsx! {
        div {
            class: {
                format_args!("discord-bottom-pane {}", if show_discord_server_pane() { "show" } else { "" })
            },
            h2 { class: "discord-centered-heading", "Discord" }
            if !discord_guilds().is_null() {
                // Render the discord_guilds data
                ul {
                    class: "guild-list",
                    for guild in discord_guilds_array {
                        li {
                            class: "guild-item",
                            button {
                                class: "guild-button",  // You can style this button as you like in CSS
                                onclick: move |_| handle_get_channels(guild["id"].as_str().unwrap().to_string(), Arc::clone(&user())) ,
                                {guild["name"].as_str().unwrap_or("Unknown Guild")}
                            }
                        }
                    }
                }
            } else {
                p { "No discord_guilds available." }
            }
            ChannelList {
                user: user.clone(),
                channels: channels.clone(),
                show_channel_pane: show_channel_pane.clone(),
                show_discord_server_pane: show_discord_server_pane.clone()
            }
        }
    }
}

#[component]
fn ChannelList(user: Signal<Arc<Mutex<User>>>, channels: Signal<Option<Value>>, show_channel_pane: Signal<bool>, show_discord_server_pane: Signal<bool>) -> Element {
    let channels_array = channels()?.as_array().unwrap_or(&vec![]).clone();
    let mut messages = use_signal(|| None::<Value>);
    let mut fetch_error = use_signal(|| None::<String>);
    let mut show_channel_messages_pane = use_signal(|| false);
    let mut current_channel_id = use_signal(|| " ".to_string());
   

    // Fetch the channels for the selected guild
    let handle_get_channel_messages = move |channel_id: String, user_lock_api: Arc<Mutex<User>>| {
        let channel_id_clone = channel_id.clone();

        block_on(async move {
            let user_lock_api = user_lock_api.lock().await;
            let discord_token = user_lock_api.discord.token.clone();
            match get_messages(discord_token.to_string(), channel_id).await {
                Ok(messages_data) => {
                    messages.set(Some(messages_data));
                    current_channel_id.set(channel_id_clone);
                    show_channel_messages_pane.set(true);
                }
                Err(e) => {
                    fetch_error.set(Some(e.to_string()));
                }
            }
        });

    };

    rsx! {
        div {
            class: {
                format_args!("channel-list-pane {}", if show_channel_pane() && show_discord_server_pane() { "show" } else { "" })
            },
            h2 { class: "discord-centered-heading", "Channels" }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| { show_channel_pane.set(false);},
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    view_box: "0 0 24 24",
                    width: "30", // Adjust size as needed
                    height: "30", // Adjust size as needed
                    path {
                        d: "M18 6 L6 18 M6 6 L18 18", // This path describes a close icon (X)
                        fill: "none",
                        stroke: "#f5f5f5", // Change stroke color as needed
                        stroke_width: "2" // Adjust stroke width
                    }
                }
            }
            if !channels()?.is_null() {
                ul {
                    class: "channel-list",
                    for channel in channels_array {
                        li {
                            class: "channel-item",
                            button {
                                class: "channel-button",
                                onclick: move |_| {handle_get_channel_messages(channel["id"].as_str().unwrap().to_string(), Arc::clone(&user()))},
                                {channel["name"].as_str().unwrap_or("Unknown Channel")}
                            }
                        }
                    }
                }
            }
            ChannelMessages {
                user: user.clone(),
                messages: messages.clone(),
                show_channel_messages_pane: show_channel_messages_pane.clone(),
                current_channel_id: current_channel_id,
                show_discord_server_pane: show_discord_server_pane.clone()
            }
        }
    }
}

#[derive(Debug, Clone)]
struct EmptyStruct {} // Empty struct to use for coroutines (when you don't need to send anything into the coroutine)

#[component]
fn ChannelMessages(user: Signal<Arc<Mutex<User>>>, messages: Signal<Option<Value>>, show_channel_messages_pane: Signal<bool>, current_channel_id: Signal<String>,  show_discord_server_pane: Signal<bool>) -> Element {
    let mut send_error = use_signal(|| None::<String>);
    let mut message_input = use_signal(|| "".to_string());
    let mut attachment_name = use_signal(|| "".to_string());
    let mut attachment_input = use_signal(|| Vec::new());
    let user_lock_api = Arc::clone(&user());

    let handle_send_message = move |user_lock_api: Arc<Mutex<User>>| {
        block_on(async move {
            let user_lock_api = user_lock_api.lock().await;
            let discord_token = user_lock_api.discord.token.clone();
            
            // Check if the attachment_input contains data
            if !attachment_input.is_empty() {
                // Attachment exists, send message with attachment
                match send_message_attachment(
                    discord_token.to_string(),
                    current_channel_id.to_string(),
                    message_input.to_string(),
                    attachment_input(),
                    attachment_name.to_string()
                ).await {
                    Ok(_send_response) => {
                        info!("Message with attachment sent successfully");
                        // Clear the attachment input and name after sending the message
                        attachment_input.set(Vec::new()); // Assuming attachment_input is a Vec<u8> signal
                        attachment_name.set(String::new()); // Assuming attachment_name is a String signal
                    }
                    Err(e) => {
                        send_error.set(Some(e.to_string()));
                        info!("Message with attachment send failed: {}", e);
                    }
                }
            } else {
                // No attachment, send a text message
                match send_message(
                    discord_token.to_string(),
                    current_channel_id.to_string(),
                    message_input.to_string()
                ).await {
                    Ok(_send_response) => {
                        info!("Message sent successfully");
                    }
                    Err(e) => {
                        send_error.set(Some(e.to_string()));
                        info!("Message send failed: {}", e);
                    }
                }
            }
    
            // Fetch messages regardless of success or failure in sending the message
            match get_messages(discord_token.to_string(), current_channel_id.to_string()).await {
                Ok(send_response) => {
                    messages.set(Some(send_response));
                    info!("Messages update successful");
                }
                Err(e) => {
                    send_error.set(Some(e.to_string()));
                    info!("Messages update failed: {}", e);
                }
            }
        });
    };

    // Coroutine for fetching messages periodically
    let _fetch_messages = use_coroutine::<EmptyStruct,_,_>(|_rx| {
        async move {
            loop {
                // Wait for 5 seconds before fetching new messages
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                // Fetch updated messages
                let user_lock_api = user_lock_api.lock().await;
                let discord_token = user_lock_api.discord.token.clone();
                let discord_token_clone = discord_token.to_owned();
                let current_channel_id_clone = current_channel_id.to_owned();
                let mut messages_clone = messages.to_owned();

                match get_messages(discord_token_clone.to_string(), current_channel_id_clone.to_string()).await {
                    Ok(updated_messages) => {
                        messages_clone.set(Some(updated_messages)); // Update messages with the latest data
                    }
                    Err(e) => {
                        info!("Failed to fetch updated messages: {}", e);
                    }
                }
            }
        }
    });



    rsx! {
        div {
            class: {
                format_args!("channel-messages-list-pane {}", if show_channel_messages_pane() && show_discord_server_pane() { "show" } else { "" })
            },
            h2 {
                class: "discord-centered-heading",
                "Messages"
            }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| { show_channel_messages_pane.set(false);},
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    view_box: "0 0 24 24",
                    width: "30", // Adjust size as needed
                    height: "30", // Adjust size as needed
                    path {
                        d: "M18 6 L6 18 M6 6 L18 18", // This path describes a close icon (X)
                        fill: "none",
                        stroke: "#f5f5f5", // Change stroke color as needed
                        stroke_width: "2" // Adjust stroke width
                    }
                }
            }
            if let Some(messages_data) = messages() {
                ul {
                    class: "messages-list",
                    for message in messages_data.as_array().unwrap_or(&vec![]) {
                        li {
                            class: "messages-item",
                            div {
                                class: "message-header",
                                span {
                                    class: "message-username",
                                    {message["author"]["username"].as_str().unwrap_or("Unknown User")}
                                }
                                span {
                                    class: "message-date",
                                    {format_timestamp(message["timestamp"].as_str().unwrap_or(""))}
                                }
                            }
                            div {
                                class: "message-content",
                                // Display the message content
                                {message["content"].as_str().unwrap_or("Failed to display message.")}
            
                                // Check if the message has attachments and render them
                                if let Some(attachments) = message["attachments"].as_array() {
                                    for attachment in attachments {
                                        if let Some(content_type) = attachment["content_type"].as_str() {
                                            if content_type.starts_with("image/") {
                                                // Display image attachments
                                                if let Some(url) = attachment["url"].as_str() {
                                                    img {
                                                        src: "{url}",
                                                        style: "height: 30vh; display: block; margin-top: 10px;"
                                                    }
                                                }
                                            } else if content_type.starts_with("video/") {
                                                // Display video attachments
                                                if let Some(url) = attachment["url"].as_str() {
                                                    video {
                                                        src: "{url}",
                                                        controls: true,    // Enable controls
                                                        autoplay: true,    // Enable autoplay
                                                        muted: true,   
                                                        height: "30%", // Adjust width as needed
                                                        style: "display: block; margin: 10px auto;", // Center the video if needed
                                                    }
                                                }
                                            } else if content_type.starts_with("audio/") {
                                                // Display audio attachments
                                                if let Some(url) = attachment["url"].as_str() {
                                                    audio {
                                                        src: "{url}",
                                                        controls: true,    // Enable controls
                                                        autoplay: false,   // Set to true if you want autoplay
                                                        style: "display: block; margin: 10px auto;", // Center the audio if needed
                                                        // Fallback message if the audio cannot be loaded
                                                        p { "Your browser does not support the audio tag." }
                                                    }
                                                }
                                            } else {
                                                // Display the file name and a download icon for other types of files
                                                if let Some(filename) = attachment["filename"].as_str() {
                                                    if let Some(url) = attachment["url"].as_str() {
                                                        div {
                                                            style: "display: flex; align-items: center; margin: 10px auto;", // Center and align the content
                                                            p {
                                                                "{filename}" // Display the file name
                                                            }
                                                            a {
                                                                href: "{url}",
                                                                download: true,   // Enable file download
                                                                style: "margin-left: 10px;",  // Space between file name and icon
                                                                // You can add a download icon using Unicode or an image tag
                                                                img {
                                                                    src: "attachmenticon.png", // Replace with your download icon path
                                                                    width: "80px",
                                                                    alt: "Download"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div {
                    class: "message-input-container",
                    input {
                        class: "message-input-box",
                        value: "{message_input}",
                        placeholder: "Enter your message.",
                        oninput: move |event| message_input.set(event.value())
                    }
                    div {
                        class: "attachment-container",
                        input {
                            // tell the input to pick a file
                            r#type: "file",
                            // list the accepted extensions
                            accept: "",
                            // pick multiple files
                            multiple: false,
                            style: "width:70px;color:transparent",
                            onchange: move |evt| {
                                async move {
                                    if let Some(file_engine) = evt.files() {
                                        let files = file_engine.files();
                                        for file_name in &files {
                                            if let Some(file) = file_engine.read_file(file_name).await
                                            {
                                                let mut attachment = attachment_input.write();
                                                attachment.extend(file);
                                                attachment_name.set(file_name.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div {
                            class: "file-name-display",
                            // Use the attachment_name signal to display the selected file name
                            if attachment_name().is_empty() {
                                "No file chosen"
                            } else {
                                "Selected file: {attachment_name}"
                            }
                        }
                    }
                    button {  
                        class: "send-button", 
                        onclick: move |_| handle_send_message(Arc::clone(&user())),
                        // Include the SVG and the button text
                        div {
                            // Insert the SVG directly
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "16",
                                height: "16",
                                fill: "currentColor",
                                class: "bi bi-send",
                                view_box: "0 0 16 16",
                                path {
                                    d: "M15.854.146a.5.5 0 0 1 .11.54l-5.819 14.547a.75.75 0 0 1-1.329.124l-3.178-4.995L.643 7.184a.75.75 0 0 1 .124-1.33L15.314.037a.5.5 0 0 1 .54.11ZM6.636 10.07l2.761 4.338L14.13 2.576zm6.787-8.201L1.591 6.602l4.339 2.76z"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn format_timestamp(timestamp: &str) -> String {
    // Parse the timestamp string into a DateTime object
    let parsed_timestamp = DateTime::parse_from_rfc3339(timestamp).unwrap_or_else(|_| Utc::now().into());
    
    // Format the date into a readable format, e.g., "Sep 26, 2024 12:45 PM"
    parsed_timestamp.format("%b %d, %Y %I:%M %p").to_string()
}
