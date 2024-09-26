#![allow(non_snake_case)]

mod discord_api;

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use futures::executor::block_on;
use serde_json::Value;
use tokio::time;
use futures_util::StreamExt;


#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");

    let cfg = dioxus::desktop::Config::new()
        .with_custom_head(r#"<link rel="stylesheet" href="tailwind.css">"#.to_string());
    LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}

#[component]
pub fn App() -> Element {
    rsx! { Router::<Route> {} }
}

#[component]
fn Home() -> Element {
    let mut show_login_pane = use_signal(|| false);
    let mut show_discord_login_pane = use_signal(|| false);
    let mut show_discord_server_pane = use_signal(|| false);
    let mut discord_token = use_signal(|| "".to_string());
    let mut discord_guilds = use_signal(|| Value::Null);

    let handle_discord_click = move |_| {
        if discord_token.to_string() == "" {
            show_login_pane.set(!show_login_pane()); 
            show_discord_login_pane.set(!show_discord_login_pane());
        }
        else {
            show_discord_server_pane.set(!show_discord_server_pane());
        }
    };

    rsx! {
        div {
            class: "main-container",
            style: "display: flex; height: 100vh;",

            // Left vertical bar
            div {
                class: "vertical-bar",
                img {
                    src: "assets/discord_logo.png",
                    alt: "Discord Logo",
                    width: "50px",
                    height: "50px",
                    style: "cursor: pointer;",
                    onclick: handle_discord_click,
                }
            }

            // Main content area
            div {
                class: "main-content",
                style: "flex: 1; padding-left: 20px;",
                h1 { style: "color: white;", "Welcome to Lounge Lizard!" }
                h2 { style: "color: white;", "Please select a service to continue." }

                // Sliding login pane
                div {
                    class: {
                        format_args!("login-pane {}", if show_login_pane() { "show" } else { "" })
                    },
                    DiscordLogin { 
                        show_login_pane: show_login_pane.clone(), 
                        show_discord_login_pane: show_discord_login_pane.clone(),
                        show_discord_server_pane: show_discord_server_pane.clone(), 
                        discord_token: discord_token.clone(),
                        discord_guilds: discord_guilds.clone(),
                    }, 
                }

                // Bottom pane for servers
                DiscordBottomPane { 
                    show_discord_server_pane: show_discord_server_pane.clone(),
                    discord_guilds: discord_guilds.clone(),
                    discord_token: discord_token.clone()
                }, 
            }
        }
    }
}


#[component]
fn DiscordLogin(show_login_pane: Signal<bool>, show_discord_login_pane: Signal<bool>, show_discord_server_pane: Signal<bool>, discord_token: Signal<String>, discord_guilds: Signal<Value>) -> Element {
    let mut username = use_signal(|| "example@gmail.com".to_string());
    let mut password = use_signal(|| "password".to_string());

    let mut login_error = use_signal(|| None::<String>);

    let handle_login = move |_| {
        let username = username.clone();
        let password = password.clone();

        block_on(async move {
            match discord_api::login_request(username.to_string(), password.to_string()).await {
                Ok((user_id, auth_discord_token)) => {
                    discord_token.set(auth_discord_token); // Call the success handler
                    show_login_pane.set(false);
                    show_discord_login_pane.set(false);
                    show_discord_server_pane.set(true);
                    info!("Login successful");
                }
                Err(e) => {
                    login_error.set(Some(e.to_string()));
                    info!("Login failed: {}", e);
                }
            }
        });

        block_on(async move {
            match discord_api::get_guilds(discord_token.to_string()).await {
                Ok((discord_guilds_response)) => {
                    discord_guilds.set(discord_guilds_response); // Call the success handler
                    info!("discord_guilds get successful");
                }
                Err(e) => {
                    login_error.set(Some(e.to_string()));
                    info!("discord_guilds get failed: {}", e);
                }
            }
        });
    };

    rsx! {
        div {
            class: format_args!("discord-login {}", if show_discord_login_pane() { "visible" } else { "" }),
            img {
                src: "assets/discord_logo.png",
                alt: "Discord Logo",
                width: "50px",
                height: "50px",
            }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: red; color: white;",
                onclick: move |_| {show_login_pane.set(false); show_discord_login_pane.set(false)},
                "X"
            }
            input {
                class: "login-input",
                value: "{username}",
                oninput: move |event| username.set(event.value())
            }
            input {
                class: "login-input",
                r#type: "password",
                value: "{password}",
                oninput: move |event| password.set(event.value())
            }
            button { 
                class: "login-button",
                onclick: handle_login, "Login" 
            }

            if let Some(error) = login_error() {
                p { "Login failed: {error}" }
            }
        }
    }
}



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
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; color: white;",
                onclick: move |_| show_channel_pane.set(false),
                "X"
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
    let mut message_input = use_signal(|| "Enter your message here. ".to_string());

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
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; color: white;",
                onclick: move |_| show_channel_messages_pane.set(false),
                "X"
            }
            if let Some(messages_data) = messages() {
                ul {
                    class: "messages-list",
                    for message in messages_data.as_array().unwrap_or(&vec![]) {
                        li {
                            class: "messages-item",
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
                        oninput: move |event| message_input.set(event.value())
                    }
                    button {  
                        class: "send-button", 
                        onclick: handle_send_message, "Send message" 
                    }
                }
            }
        }
    }
}