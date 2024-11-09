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
    let mut show_channel_messages_pane = use_signal(|| false);
    let mut show_dm_channel_pane = use_signal(|| false);
    let mut show_dm_channel_messages_pane = use_signal(|| false);

    // Fetch the channels for the selected guild
    let handle_get_channels = move |guild_id: String, user_lock_api: Arc<Mutex<User>>| {
        block_on(async move {
            // Attempt to acquire the lock without blocking
            if let Ok(user_lock_api) = user_lock_api.try_lock() {
                let discord_token = user_lock_api.discord.token.clone();
                
                match get_guild_channels(discord_token, guild_id).await {
                    Ok(channels_data) => {
                        channels.set(Some(channels_data));
                        show_channel_pane.set(true);
                    }
                    Err(e) => {
                        fetch_error.set(Some(e.to_string()));
                        info!("Failed to fetch channels for guild");
                    }
                }
            } else {
                // Log if the lock could not be acquired
                info!("Unable to acquire user lock; skipping fetch for guild {}.", guild_id);
            }
        });
    };

    // Fetch the channels IDs for direct messages if clicked
    let handle_get_dm_channels = move |user_lock_api: Arc<Mutex<User>>| {
        block_on(async move {
            // Attempt to acquire the lock without blocking
            if let Ok(user_lock_api) = user_lock_api.try_lock() {
                let discord_token = user_lock_api.discord.token.clone();
                
                match get_channels(discord_token).await {
                    Ok(channels_data) => {
                        channels.set(Some(channels_data));
                        show_dm_channel_pane.set(true);
                    }
                    Err(e) => {
                        fetch_error.set(Some(e.to_string()));
                        info!("Failed to fetch channels for guild");
                    }
                }
            } else {
                // Log if the lock could not be acquired
                info!("Unable to acquire user lock; skipping fetch for dms.");
            }
        });
    };


    rsx! {
        div {
            class: {
                format_args!("discord-bottom-pane {}", if show_discord_server_pane() { "show" } else { "" })
            },
            div {
                // Make the div take up the full width and be clickable
                style: "width: 100%; cursor: pointer; padding: 10px 20px; text-align: center;",
                onclick: move |_| { 
                    if show_channel_pane() {
                        show_channel_pane.set(false);
                        show_channel_messages_pane.set(false);
                    }
                    else if show_dm_channel_pane() {
                        show_dm_channel_pane.set(false);
                        show_dm_channel_messages_pane.set(false);
                    }
                    else {
                        show_discord_server_pane.set(false); 
                    }
                   
                },
                h2 { class: "discord-heading", "Discord" }
            }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| { show_discord_server_pane.set(false);},
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    view_box: "0 0 24 24",
                    width: "24", // Adjust size as needed
                    height: "24", // Adjust size as needed
                    path {
                        d: "M18 6 L6 18 M6 6 L18 18", // This path describes a close icon (X)
                        fill: "none",
                        stroke: "#f5f5f5", // Change stroke color as needed
                        stroke_width: "2" // Adjust stroke width
                    }
                }
            }
            if !discord_guilds().is_null() {
                // Render the discord_guilds data
                ul {
                    class: "guild-list",
                    li {
                        class: "guild-item",
                        button {
                            class: "guild-button",  // You can style this button as you like in CSS
                            onclick: move |_| handle_get_dm_channels(Arc::clone(&user())) ,
                            {"Direct Messages"}
                        }
                    }
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
                show_discord_server_pane: show_discord_server_pane.clone(),
                show_channel_messages_pane: show_channel_messages_pane.clone()
            }
            DMChannelList {
                user: user.clone(),
                channels: channels.clone(),
                show_channel_pane: show_dm_channel_pane.clone(),
                show_discord_server_pane: show_discord_server_pane.clone(),
                show_dm_channel_messages_pane: show_dm_channel_messages_pane.clone()
            }
        }
    }
}

#[component]
fn ChannelList(user: Signal<Arc<Mutex<User>>>, channels: Signal<Option<Value>>, show_channel_pane: Signal<bool>, show_discord_server_pane: Signal<bool>, show_channel_messages_pane: Signal<bool>) -> Element {
    let channels_array = channels()?.as_array().unwrap_or(&vec![]).clone();
    let mut messages = use_signal(|| None::<Value>);
    let mut fetch_error = use_signal(|| None::<String>);
    let mut current_channel_id = use_signal(|| " ".to_string());
   

    // Fetch the channels for the selected guild
    let handle_get_channel_messages = move |channel_id: String, user_lock_api: Arc<Mutex<User>>| {
        let channel_id_clone = channel_id.clone();

        block_on(async move {
            // Attempt to acquire the lock without blocking
            if let Ok(user_lock_api) = user_lock_api.try_lock() {
                let discord_token = user_lock_api.discord.token.clone();
                
                match get_messages(discord_token.to_string(), channel_id).await {
                    Ok(messages_data) => {
                        messages.set(Some(messages_data));
                        current_channel_id.set(channel_id_clone);
                        show_channel_messages_pane.set(true);
                    }
                    Err(e) => {
                        fetch_error.set(Some(e.to_string()));
                        info!("Failed to fetch messages for channel {}: {}", channel_id_clone, e);
                    }
                }
            } else {
                // Log if the lock could not be acquired
                info!("Unable to acquire user lock; skipping fetch for channel {}.", channel_id_clone);
            }
        });
    };


    rsx! {
        div {
            class: {
                format_args!("channel-list-pane {}", if show_channel_pane() && show_discord_server_pane() { "show" } else { "" })
            },
            div {
                // Make the div take up the full width and be clickable
                style: "width: 100%; cursor: pointer; padding: 10px 20px; text-align: center;",
                onclick: move |_| { 
                    if show_channel_messages_pane() {
                        show_channel_messages_pane.set(false);
                    }
                    else {
                        show_channel_pane.set(false); 
                        show_channel_messages_pane.set(false);
                    }
                },
                h2 { class: "discord-heading", "Channels" }
            }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| { show_channel_pane.set(false); show_channel_messages_pane.set(false);},
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    view_box: "0 0 24 24",
                    width: "24", // Adjust size as needed
                    height: "24", // Adjust size as needed
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

#[component]
fn DMChannelList(user: Signal<Arc<Mutex<User>>>, channels: Signal<Option<Value>>, show_channel_pane: Signal<bool>, show_discord_server_pane: Signal<bool>, show_dm_channel_messages_pane: Signal<bool>) -> Element {
    let channels_array = channels()?.as_array().unwrap_or(&vec![]).clone();
    let mut messages = use_signal(|| None::<Value>);
    let mut fetch_error = use_signal(|| None::<String>);
    let mut current_channel_id = use_signal(|| " ".to_string());
   

    // Fetch the channels for the selected guild
    let handle_get_channel_messages = move |channel_id: String, user_lock_api: Arc<Mutex<User>>| {
        let channel_id_clone = channel_id.clone();

        block_on(async move {
            // Attempt to acquire the lock without blocking
            if let Ok(user_lock_api) = user_lock_api.try_lock() {
                let discord_token = user_lock_api.discord.token.clone();
                
                match get_messages(discord_token.to_string(), channel_id).await {
                    Ok(messages_data) => {
                        messages.set(Some(messages_data));
                        current_channel_id.set(channel_id_clone);
                        show_dm_channel_messages_pane.set(true);
                    }
                    Err(e) => {
                        fetch_error.set(Some(e.to_string()));
                        info!("Failed to fetch messages for channel {}: {}", channel_id_clone, e);
                    }
                }
            } else {
                // Log if the lock could not be acquired
                info!("Unable to acquire user lock; skipping fetch for channel {}.", channel_id_clone);
            }
        });
    };


    rsx! {
        div {
            class: {
                format_args!("channel-list-pane {}", if show_channel_pane() && show_discord_server_pane() { "show" } else { "" })
            },
            div {
                // Make the div take up the full width and be clickable
                style: "width: 100%; cursor: pointer; padding: 10px 20px; text-align: center;",
                onclick: move |_| { 
                    if show_dm_channel_messages_pane() {
                        show_dm_channel_messages_pane.set(false);
                    }
                    else {
                        show_channel_pane.set(false); 
                        show_dm_channel_messages_pane.set(false);
                    }
                },
                h2 { class: "discord-heading", "DM Channels" }
            }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| { show_channel_pane.set(false); show_dm_channel_messages_pane.set(false);},
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    view_box: "0 0 24 24",
                    width: "24", // Adjust size as needed
                    height: "24", // Adjust size as needed
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
                                {
                                    // Use `global_name` if it exists; otherwise, use `username`
                                    channel["recipients"][0]["global_name"]
                                        .as_str()
                                        .unwrap_or_else(|| channel["recipients"][0]["username"].as_str().unwrap_or("Unknown User"))
                                }
                            }
                        }
                    }
                }
            }
            ChannelMessages {
                user: user.clone(),
                messages: messages.clone(),
                show_channel_messages_pane: show_dm_channel_messages_pane.clone(),
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
    let mut message_id_input = use_signal(|| "".to_string());
    let mut reaction_input = use_signal(|| "".to_string());
    let mut attachment_name = use_signal(|| "".to_string());
    let mut attachment_input = use_signal(|| Vec::new());
    let user_lock_api = Arc::clone(&user());

    let handle_send_message = move |user_lock_api: Arc<Mutex<User>>| {
        block_on(async move {
            // Attempt to acquire the lock without blocking.
            if let Ok(user_lock_api) = user_lock_api.try_lock() {
                let discord_token = user_lock_api.discord.token.clone();
    
                // Check if the attachment_input contains data
                if !attachment_input.is_empty() {
                    // Attachment exists, send message with attachment
                    match send_message_attachment(
                        discord_token.to_string(),
                        current_channel_id.to_string(),
                        message_input.to_string(),
                        attachment_input(),
                        attachment_name.to_string(),
                    )
                    .await
                    {
                        Ok(_send_response) => {
                            info!("Message with attachment sent successfully");
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
                        message_input.to_string(),
                    )
                    .await
                    {
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
    
                // Clear the attachment input and name after sending the message
                attachment_input.set(Vec::new()); // Assuming attachment_input is a Vec<u8> signal
                attachment_name.set(String::new()); // Assuming attachment_name is a String signal
            } else {
                // Handle the case where the lock could not be acquired
                info!("Failed to acquire user lock; skipping message send.");
            }
        });
    };
    

    let handle_send_reaction = move |user_lock_api: Arc<Mutex<User>>| {
        block_on(async move {
            // Attempt to acquire the lock without blocking.
            if let Ok(user_lock_api) = user_lock_api.try_lock() {
                let discord_token = user_lock_api.discord.token.clone();
                println!("reaction from handler is: {}", reaction_input.to_string());
    
                // Send a reaction
                match send_reaction(
                    discord_token.to_string(),
                    current_channel_id.to_string(),
                    message_id_input.to_string(),
                    reaction_input.to_string(),
                )
                .await
                {
                    Ok(_send_response) => {
                        info!("Reaction sent successfully");
                    }
                    Err(e) => {
                        send_error.set(Some(e.to_string()));
                        info!("Message send failed: {}", e);
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
    
                // Clear emoji and message ID after sending reaction and fetching messages
                reaction_input.set(String::new());
                message_id_input.set(String::new());
            } else {
                // Handle the case where the lock could not be acquired
                info!("Failed to acquire user lock; skipping reaction send.");
            }
        });
    };
    

    // Coroutine for fetching messages periodically
    let _fetch_messages = use_coroutine::<EmptyStruct, _, _>(|_rx| {
        let user_lock_api = user_lock_api.clone();
        let current_channel_id = current_channel_id.clone();
        let mut messages = messages.clone();

        async move {
            loop {
                // Wait for 5 seconds before fetching new messages
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                if show_channel_messages_pane() && show_discord_server_pane() {
                    // Attempt to acquire the lock without blocking
                    if let Ok(user_lock_api) = user_lock_api.try_lock() {
                        let discord_token = user_lock_api.discord.token.clone();
                        
                        match get_messages(discord_token.to_string(), current_channel_id.to_string()).await {
                            Ok(updated_messages) => {
                                messages.set(Some(updated_messages)); // Update messages with the latest data
                                info!("Messages updated successfully.");
                            }
                            Err(e) => {
                                info!("Failed to fetch updated messages: {}", e);
                            }
                        }
                    } else {
                        // Log if the lock could not be acquired
                        info!("Unable to acquire user lock; skipping message fetch for this cycle.");
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
            div {
                // Make the div take up the full width and be clickable
                style: "width: 100%; cursor: pointer; padding: 10px 20px; text-align: center;",
                onclick: move |_| { show_channel_messages_pane.set(false); },
                h2 {class: "discord-heading", "Messages"}
            }
            
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| { show_channel_messages_pane.set(false);},
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    view_box: "0 0 24 24",
                    width: "24", // Adjust size as needed
                    height: "24", // Adjust size as needed
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
                            onclick: {
                                let current_message_id = message["id"].to_string().clone();
                                move |_| {
                                    message_id_input.set(current_message_id.clone());
                                }
                            },
                            div {
                                class: "message-header",
                                img {
                                    class: "message-avatar",
                                    src: { 
                                        if let Some(avatar) = message["author"]["avatar"].as_str() {
                                            format!("https://cdn.discordapp.com/avatars/{}/{}.webp", message["author"]["id"].as_str().unwrap(), avatar)
                                        } else {
                                            "assets/defaultpfp.png".to_string() // Path to your default avatar image
                                        }
                                    },
                                    alt: "User Avatar"
                                }
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
                                div {
                                    style: "display: flex; justify-content: center; align-items: center;",
                                    if let Some(attachments) = message["attachments"].as_array() {
                                        for attachment in attachments {
                                            if let Some(content_type) = attachment["content_type"].as_str() {
                                                if content_type.starts_with("image/") {
                                                    // Display image attachments
                                                    if let Some(url) = attachment["url"].as_str() {
                                                        img {
                                                            src: "{url}",
                                                            style: "max-height: 25vh; display: block; margin-top: 10px;"
                                                        }
                                                    }
                                                } else if content_type.starts_with("video/") {
                                                    // Display video attachments
                                                    if let Some(url) = attachment["url"].as_str() {
                                                        video {
                                                            src: "{url}",
                                                            controls: true,    // Enable controls
                                                            autoplay: false,    // Enable autoplay
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
                                                            a {
                                                                href: "{url}", // Make the whole div clickable
                                                                class: "file-attachment-other-container", // Center and align the content
                                                                svg {
                                                                    view_box: "0 0 24 24",
                                                                    width: "40px",
                                                                    height: "40px",
                                                                    fill: "none",
                                                                    xmlns: "http://www.w3.org/2000/svg",
                                                                    stroke: "#ffffff",
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
                                                                            d: "M19 9V17.8C19 18.9201 19 19.4802 18.782 19.908C18.5903 20.2843 18.2843 20.5903 17.908 20.782C17.4802 21 16.9201 21 15.8 21H8.2C7.07989 21 6.51984 21 6.09202 20.782C5.71569 20.5903 5.40973 20.2843 5.21799 19.908C5 19.4802 5 18.9201 5 17.8V6.2C5 5.07989 5 4.51984 5.21799 4.09202C5.40973 3.71569 5.71569 3.40973 6.09202 3.21799C6.51984 3 7.0799 3 8.2 3H13M19 9L13 3M19 9H14C13.4477 9 13 8.55228 13 8V3",
                                                                            stroke: "#f5f5f5",
                                                                            stroke_width: "1.176",
                                                                            stroke_linecap: "round",
                                                                            stroke_linejoin: "round",
                                                                        }
                                                                    }
                                                                }
                                                                p {
                                                                    style: "margin-left: 5px; text-align: center;", // Center text
                                                                    "{filename}" // Display the file name
                                                                }
                                                                svg {
                                                                    style: "margin-left: 20px; margin-right: 5px",
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
                                                                            d: "M8 22.0002H16C18.8284 22.0002 20.2426 22.0002 21.1213 21.1215C22 20.2429 22 18.8286 22 16.0002V15.0002C22 12.1718 22 10.7576 21.1213 9.8789C20.3529 9.11051 19.175 9.01406 17 9.00195M7 9.00195C4.82497 9.01406 3.64706 9.11051 2.87868 9.87889C2 10.7576 2 12.1718 2 15.0002L2 16.0002C2 18.8286 2 20.2429 2.87868 21.1215C3.17848 21.4213 3.54062 21.6188 4 21.749",
                                                                            stroke: "#f5f5f5",
                                                                            stroke_width: "1.5",
                                                                            stroke_linecap: "round",
                                                                        }
                                                                        path {
                                                                            d: "M12 2L12 15M12 15L9 11.5M12 15L15 11.5",
                                                                            stroke: "#f5f5f5",
                                                                            stroke_width: "1.5",
                                                                            stroke_linecap: "round",
                                                                            stroke_linejoin: "round",
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
                                class: "reactions",
                                if let Some(reactions) = message["reactions"].as_array() {
                                    for reaction in reactions {
                                        span {
                                            class: "reaction",
                                            onclick: { 
                                                let reaction_emoji = reaction["emoji"]["name"].to_string().clone();
                                                let current_message_id = message["id"].to_string().clone();
                                                // println!("reaction input from div {}", reaction_input);
                                                move |_|{
                                                    reaction_input.set(reaction_emoji.clone());
                                                    message_id_input.set(current_message_id.clone());
                                                    handle_send_reaction(Arc::clone(&user())) 
                                                }
                                            },
                                            {
                                                reaction["emoji"]["name"].as_str().unwrap_or("")
                                            }
                                            {
                                                {" ".to_string() + &reaction["count"].as_u64().unwrap_or(0).to_string()}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div {
                    div {
                        class: format_args!("file-name-display {}", if !attachment_name().is_empty() { "show" } else { "" }),

                        // Close button
                        button {
                            class: "file-close-button",
                            onclick: move |_| {
                                attachment_input.set(Vec::new()); 
                                attachment_name.set(String::new());
                            },
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 24 24",
                                width: "20", // Adjust size as needed
                                height: "20", // Adjust size as needed
                                path {
                                    d: "M18 6 L6 18 M6 6 L18 18", // This path describes a close icon (X)
                                    fill: "none",
                                    stroke: "#141414", // Change stroke color as needed
                                    stroke_width: "2" // Adjust stroke width
                                }
                            }
                        }
                        
                        // Use the attachment_name signal to display the selected file name
                        if attachment_name().is_empty() {
                            "No attachment added."
                        } else {
                            "{attachment_name}"
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
                            // Paperclip icon button
                            label {   
                                r#for: "file-input",
                                class: "attachment-button",
                                svg {
                                    view_box: "0 0 24 24",
                                    width: "30px",
                                    height: "30px",
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
                                            if let Some(file_engine) = evt.files() {
                                                let files = file_engine.files();
                                                for file_name in &files {
                                                    if let Some(file) = file_engine.read_file(file_name).await {
                                                        attachment_input.set(Vec::new()); // clear before writing
                                                        attachment_name.set(String::new()); // clear before writing
                                                        let mut attachment = attachment_input.write();
                                                        attachment.extend(file);
                                                        
                                                        // Use std::path::Path to extract the file name without the path
                                                        let base_name = std::path::Path::new(file_name).file_name()
                                                        .and_then(|name| name.to_str())
                                                        .unwrap_or("Unknown File");
                                                        attachment_name.set(base_name.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        button {  
                            class: "send-button", 
                            onclick: move |_| handle_send_message(Arc::clone(&user())),
                            div {
                                svg {
                                    view_box: "0 0 24 24",
                                    width: "30px",
                                    height: "30px",
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
                                            fill_rule: "evenodd",
                                            clip_rule: "evenodd",
                                            d: "M3.3938 2.20468C3.70395 1.96828 4.12324 1.93374 4.4679 2.1162L21.4679 11.1162C21.7953 11.2895 22 11.6296 22 12C22 12.3704 21.7953 12.7105 21.4679 12.8838L4.4679 21.8838C4.12324 22.0662 3.70395 22.0317 3.3938 21.7953C3.08365 21.5589 2.93922 21.1637 3.02382 20.7831L4.97561 12L3.02382 3.21692C2.93922 2.83623 3.08365 2.44109 3.3938 2.20468ZM6.80218 13L5.44596 19.103L16.9739 13H6.80218ZM16.9739 11H6.80218L5.44596 4.89699L16.9739 11Z",
                                            fill: "#f5f5f5",
                                        }
                                    }           
                                }
                            }
                        }
                    }
                    div {
                        class: format_args!("reaction-picker {}", if !message_id_input().is_empty() { "show" } else { "" }),
                        // Close button
                        button {
                            style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                            onclick: move |_| {
                                message_id_input.set(String::new());
                                reaction_input.set(String::new());
                            },
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                view_box: "0 0 24 24",
                                width: "24", // Adjust size as needed
                                height: "24", // Adjust size as needed
                                path {
                                    d: "M18 6 L6 18 M6 6 L18 18", // This path describes a close icon (X)
                                    fill: "none",
                                    stroke: "#f5f5f5", // Change stroke color as needed
                                    stroke_width: "2" // Adjust stroke width
                                }
                            }
                        }

                        // Emojis
                        button {
                            class: "reaction-picker-item",
                            onclick: {
                                move |_|{
                                    reaction_input.set("👍".to_string());
                                    handle_send_reaction(Arc::clone(&user()));
                                }
                            },
                            "👍"  // Thumbs up emoji
                        }
                        button {
                            class: "reaction-picker-item",
                            onclick: move |_| {
                                reaction_input.set("👎".to_string());
                                handle_send_reaction(Arc::clone(&user()));
                            },
                            "👎"
                        }
                        button {
                            class: "reaction-picker-item",
                            onclick: move |_| {
                                reaction_input.set("❤️".to_string());
                                handle_send_reaction(Arc::clone(&user()));
                            },
                            "❤️"
                        }
                        button {
                            class: "reaction-picker-item",
                            onclick: move |_| {
                                reaction_input.set("❗".to_string());
                                handle_send_reaction(Arc::clone(&user()));
                            },
                            "❗"
                        }
                        button {
                            class: "reaction-picker-item",
                            onclick: move |_| {
                                reaction_input.set("❓".to_string());
                                handle_send_reaction(Arc::clone(&user()));
                            },
                            "❓"
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
