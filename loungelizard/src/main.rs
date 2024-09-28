#![allow(non_snake_case)]

mod discord_api;

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use futures::executor::block_on;
use serde_json::{json, Value};
use tokio::time;
use futures_util::StreamExt;
use chrono::{DateTime, Utc, NaiveDateTime};


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
        .with_custom_head(r#"<link rel="stylesheet" href="/assets/tailwind.css">"#.to_string());
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

            // Left vertical bar
            div {
                class: "vertical-bar",
                div {
                    class: {
                        format_args!("white-square {}", if show_discord_login_pane() || show_discord_server_pane() { "opaque" } else { "transparent" })
                    },
                    img {
                        src: "assets/discord_logo.png",
                        alt: "Discord Logo",
                        width: "50px",
                        height: "50px",
                        style: "cursor: pointer;",
                        onclick: handle_discord_click,
                    }
                }
            }

            // Main content area
            div {
                class: "main-content",

                h1 { 
                    class: "welcome-message", 
                    "welcome back to lounge lizard" 
                }

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
    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());

    let mut login_error = use_signal(|| None::<String>);
    let mut captcha_required = use_signal(|| false);
    let mut captcha_sitekey = use_signal(|| "".to_string());

    let mut eval = eval(r#"
        // Inject the hCaptcha API script if it's not already loaded
        if (!document.querySelector('script[src="https://hcaptcha.com/1/api.js"]')) {
            const script = document.createElement('script');
            script.src = "https://hcaptcha.com/1/api.js";
            script.async = true;
            script.defer = true;
            document.head.appendChild(script);

            // Function to render hCaptcha widget once the script is loaded
            script.onload = async function() {
                // Wait for the sitekey from Rust (via Dioxus)
                let sitekey = await dioxus.recv();
                console.log("Received sitekey: " + sitekey);

                // Render the hCaptcha widget with the received sitekey
                renderCaptchaWidget(sitekey);
            };
        } else {
            // If script is already loaded, immediately wait for the sitekey
            let sitekey = await dioxus.recv();
            console.log("Received sitekey: " + sitekey);

            // Render the hCaptcha widget with the received sitekey
            renderCaptchaWidget(sitekey);
        }

        // Function to render the hCaptcha widget
        function renderCaptchaWidget(sitekey) {
            if (document.getElementById('hcaptcha-widget')) {
                hcaptcha.render('hcaptcha-widget', {
                    sitekey: sitekey, // Dynamically set the sitekey from Rust
                    callback: function(response) {
                        // Send the CAPTCHA response token back to Rust
                        console.log("CAPTCHA response token in send: " + response);
                        dioxus.send(response);
                        console.log("CAPTCHA response sent to dioxus! ");
                    }
                });
            } else {
                console.error('CAPTCHA widget container not found.');
            }
        }

    "#);

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
                }
                Err(e) => {
                    let error_message = e.to_string(); // Create a longer-lived variable
                    if error_message.contains("captcha-required") {
                        println!("in captcha required");
                        println!("Error message: {}", error_message); // Debug print
                        // Simplified regex to capture everything after "sitekey = "
                        let re = regex::Regex::new(r"sitekey = ([^,]+), rqtoken = ([^,]+)").unwrap();

                        if let Some(captures) = re.captures(e.to_string().as_str()) {
                            if captures.len() == 3 {
                                let sitekey = captures.get(1).map_or("", |m| m.as_str()).trim();
                                let rqtoken = captures.get(2).map_or("", |m| m.as_str()).trim();
                        
                                println!("Final Captcha sitekey: {}", sitekey);
                                println!("Final Captcha rqtoken: {}", rqtoken);
                        
                                // Set the CAPTCHA flag and trigger CAPTCHA rendering
                                captcha_sitekey.set(sitekey.to_string());
                                captcha_required.set(true);
                                eval.send(sitekey.into()).unwrap();

                                // Capture the CAPTCHA response from JavaScript
                                let _future = use_resource(move || {
                                    to_owned![eval];
                                    async move {
                                        println!("Waiting for CAPTCHA token...");
                                        let response = eval.recv().await;
                                        println!("After awaiting response...");
                                        match response {
                                            Ok(token) => {
                                                let token_str = token.to_string(); // Convert the Value to a string
                                                let trimmed_token = token_str.trim_matches('"'); // Remove surrounding quotes if present
                                                
                                                println!("Received CAPTCHA token: {}", trimmed_token); // Print the parsed token
                                                match discord_api::login_request_captcha(username.to_string(), password.to_string(), trimmed_token.to_string(), rqtoken.to_string()).await {
                                                    Ok((user_id, auth_discord_token)) => {
                                                        discord_token.set(auth_discord_token); // Call the success handler
                                                        show_login_pane.set(false);
                                                        show_discord_login_pane.set(false);
                                                        show_discord_server_pane.set(true);
                                                        info!("Login with captcha successful");
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
                                                    }
                                                    Err(e) => {
                                                        println!("something went wrong in captcha login final step");
                                                        login_error.set(Some(e.to_string()));
                                                        info!("Login failed: {}", e);
                                                    }
                                                }
                                                token // This returns `serde_json::Value`
                                            },
                                            Err(e) => {
                                                println!("Error receiving CAPTCHA token: {:?}", e);
                                                // Return an empty Value or a specific error Value
                                                serde_json::Value::Null // or any valid Value indicating an error
                                            }
                                        }
                                    }
                                });

                            }
                        }
                    }
                    else {
                        println!("rqtoken and sitekey not in captcha required");
                        login_error.set(Some(e.to_string()));
                        info!("Login failed: {}", e);
                    }
                    info!("Login failed: {}", e);
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
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| { show_login_pane.set(false); show_discord_login_pane.set(false) },
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
            input {
                class: "login-input",
                value: "{username}",
                placeholder: "Username/Email",
                oninput: move |event| username.set(event.value())
            }
            input {
                class: "login-input",
                r#type: "password",
                value: "{password}",
                placeholder: "Password",
                oninput: move |event| password.set(event.value())
            }
            button { 
                class: "login-button",
                onclick: handle_login, "Login" 
            }

            //if captcha_required() {
                div { id: "hcaptcha-widget" } // This is where hCaptcha will be rendered
            //}

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