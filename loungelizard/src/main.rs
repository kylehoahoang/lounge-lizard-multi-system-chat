#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, error, Level};


use futures::executor::block_on;
use serde_json::Value;
use tokio::time;
use futures_util::StreamExt;
use chrono::{DateTime, Utc, NaiveDateTime};


// * Regular Page Routing Files 
mod pages;
use pages::Discord::*;
use pages::MSTeam::*;
use pages::Slack::*;
use pages::Home::*;

// * Login Page Routing Files
mod logins;

// * Api server files
mod api;
use api::discord_api;
use api::mongo_format::mongo_structs::*;

use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};


#[derive(Clone, Routable, Debug, PartialEq)]
enum AppRoute {
    #[route("/")]
    Home {},
    #[route("/Slack")]
    Slack {},
    #[route("/Discord")]
    Discord {},
    #[route("/MSTeams")]
    MSTeams {},
}

// Global User instance using lazy_static
lazy_static! {
    static ref GLOBAL_USER: Arc<Mutex<User>> = Arc::new(Mutex::new(User::default()));
}


fn main() {

    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting application...");


    let cfg = dioxus::desktop::Config::new()
        .with_custom_head(r#"<link rel="stylesheet" href="/assets/tailwind.css">"#.to_string());

    LaunchBuilder::desktop()
        .with_cfg(cfg)
        .launch(App);

}

#[component]
fn App() -> Element {

    // Create a global signal for the Arc<Mutex<User>> data
    let user_lock = use_signal(|| GLOBAL_USER.clone());

    provide_context(user_lock.clone());

    rsx! { Router::<AppRoute> {} }
}


// TODO Everything below could be moved
#[component]
fn DiscordBottomPane(show_discord_server_pane: Signal<bool>, discord_guilds: Signal<Value>, discord_token: Signal<String>) -> Element {
    let discord_guilds_array = discord_guilds().as_array().unwrap_or(&vec![]).clone();
    let mut channels = use_signal(|| None::<Value>);
    let mut fetch_error = use_signal(|| None::<String>);
    let mut show_channel_pane = use_signal(|| false);

    // Fetch the channels for the selected guild
    let handle_get_channels = move |guild_id: String| {
        block_on(async move {
            match discord_api::get_guild_channels(discord_token.to_string(), guild_id).await {
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
            h2 { "Discord Servers" }
            if !discord_guilds().is_null() {
                // Render the discord_guilds data
                ul {
                    class: "guild-list",
                    for guild in discord_guilds_array {
                        li {
                            class: "guild-item",
                            button {
                                class: "guild-button",  // You can style this button as you like in CSS
                                onclick: move |_| handle_get_channels(guild["id"].as_str().unwrap().to_string()) ,
                                {guild["name"].as_str().unwrap_or("Unknown Guild")}
                            }
                        }
                    }
                }
            } else {
                p { "No discord_guilds available." }
            }
            ChannelList {
                discord_token: discord_token.clone(),
                channels: channels.clone(),
                show_channel_pane: show_channel_pane.clone(),
                show_discord_server_pane: show_discord_server_pane.clone()
            }
        }
    }
}

#[component]
fn ChannelList(discord_token: Signal<String>, channels: Signal<Option<Value>>, show_channel_pane: Signal<bool>, show_discord_server_pane: Signal<bool>) -> Element {
    let channels_array = channels()?.as_array().unwrap_or(&vec![]).clone();
    let mut messages = use_signal(|| None::<Value>);
    let mut fetch_error = use_signal(|| None::<String>);
    let mut show_channel_messages_pane = use_signal(|| false);
    let mut current_channel_id = use_signal(|| " ".to_string());

    // Fetch the channels for the selected guild
    let handle_get_channel_messages = move |channel_id: String| {
        let channel_id_clone = channel_id.clone();

        block_on(async move {
            match discord_api::get_messages(discord_token.to_string(), channel_id).await {
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
            h2 { "Channels" }
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
                                onclick: move |_| {handle_get_channel_messages(channel["id"].as_str().unwrap().to_string())},
                                {channel["name"].as_str().unwrap_or("Unknown Channel")}
                            }
                        }
                    }
                }
            }
            ChannelMessages {
                discord_token: discord_token.clone(),
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
fn ChannelMessages(discord_token: Signal<String>, messages: Signal<Option<Value>>, show_channel_messages_pane: Signal<bool>, current_channel_id: Signal<String>,  show_discord_server_pane: Signal<bool>) -> Element {
    let mut send_error = use_signal(|| None::<String>);
    let mut message_input = use_signal(|| "".to_string());

    let handle_send_message = move |_| {
        block_on(async move {
            match discord_api::send_message(discord_token.to_string(), current_channel_id.to_string(), message_input.to_string()).await {
                Ok((send_response)) => {
                    info!("Message sent successfully");
                }
                Err(e) => {
                    send_error.set(Some(e.to_string()));
                    info!("Message send failed: {}", e);
                }
            }
        });

        block_on(async move {
            match discord_api::get_messages(discord_token.to_string(), current_channel_id.to_string()).await {
                Ok((send_response)) => {
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
    let _fetch_messages = use_coroutine::<EmptyStruct,_,_>(|rx| {
        async move {
            loop {
                // Wait for 5 seconds before fetching new messages
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                // Fetch updated messages
                let discord_token_clone = discord_token.to_owned();
                let current_channel_id_clone = current_channel_id.to_owned();
                let mut messages_clone = messages.to_owned();

                match discord_api::get_messages(discord_token_clone.to_string(), current_channel_id_clone.to_string()).await {
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
            h2 { "Messages" }
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
                            button {
                                class: "messages-button",
                                {message["content"].as_str().unwrap_or("Failed to display message.")}
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
                    button {  
                        class: "send-button", 
                        onclick: handle_send_message, "Send" 
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