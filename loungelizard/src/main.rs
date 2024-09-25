#![allow(non_snake_case)]

mod discord_api;

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use futures::executor::block_on;
use serde_json::Value;


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
    let mut show_server_pane = use_signal(|| false);
    let mut token = use_signal(|| "".to_string());
    let mut guilds = use_signal(|| Value::Null);

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
                    onclick: move |_| show_login_pane.set(!show_login_pane()),
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
                    Login { 
                        show_login_pane: show_login_pane.clone(), 
                        show_server_pane: show_server_pane.clone(), 
                        token: token.clone(),
                        guilds: guilds.clone(),
                    }, 
                }

                // Bottom pane for servers
                BottomPane { 
                    show_server_pane: show_server_pane.clone(),
                    guilds: guilds.clone(),
                    token: token.clone()
                }, 
            }
        }
    }
}


#[component]
fn Login(show_login_pane: Signal<bool>, show_server_pane: Signal<bool>, token: Signal<String>, guilds: Signal<Value>) -> Element {
    let mut username = use_signal(|| "example@gmail.com".to_string());
    let mut password = use_signal(|| "password".to_string());

    let mut login_error = use_signal(|| None::<String>);

    let handle_login = move |_| {
        let username = username.clone();
        let password = password.clone();

        block_on(async move {
            match discord_api::login_request(username.to_string(), password.to_string()).await {
                Ok((user_id, auth_token)) => {
                    token.set(auth_token); // Call the success handler
                    show_login_pane.set(false);
                    show_server_pane.set(true);
                    info!("Login successful");
                }
                Err(e) => {
                    login_error.set(Some(e.to_string()));
                    info!("Login failed: {}", e);
                }
            }
        });

        block_on(async move {
            match discord_api::get_guilds(token.to_string()).await {
                Ok((guilds_response)) => {
                    guilds.set(guilds_response); // Call the success handler
                    info!("Guilds get successful");
                }
                Err(e) => {
                    login_error.set(Some(e.to_string()));
                    info!("Guilds get failed: {}", e);
                }
            }
        });
    };

    rsx! {
        div {
            h1 { style: "color: white;", "Login" }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: red; color: white;",
                onclick: move |_| show_login_pane.set(false),
                "X"
            }
            input {
                value: "{username}",
                oninput: move |event| username.set(event.value())
            }
            input {
                r#type: "password",
                value: "{password}",
                oninput: move |event| password.set(event.value())
            }
            button { onclick: handle_login, "Login" }

            if let Some(error) = login_error() {
                p { "Login failed: {error}" }
            }
        }
    }
}



#[component]
fn BottomPane(show_server_pane: Signal<bool>, guilds: Signal<Value>, token: Signal<String>) -> Element {
    let guilds_array = guilds().as_array().unwrap_or(&vec![]).clone();
    let mut channels = use_signal(|| None::<Value>);
    let mut fetch_error = use_signal(|| None::<String>);
    let mut show_channel_pane = use_signal(|| false);

    // Fetch the channels for the selected guild
    let handle_get_channels = move |guild_id: String| {
        let guild_id = guild_id.clone();
        let token = token.clone();

        block_on(async move {
            match discord_api::get_guild_channels(token.to_string(), guild_id).await {
                Ok(channels_data) => {
                    channels.set(Some(channels_data));
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
                format_args!("bottom-pane {}", if show_server_pane() { "show" } else { "" })
            },
            if !guilds().is_null() {
                // Render the guilds data
                ul {
                    class: "guild-list",
                    for guild in guilds_array {
                        li {
                            class: "server-item",
                            button {
                                class: "guild-button",  // You can style this button as you like in CSS
                                onclick: move |_| handle_get_channels(guild["id"].as_str().unwrap().to_string()) ,
                                {guild["name"].as_str().unwrap_or("Unknown Guild")}
                            }
                        }
                    }
                }
            } else {
                p { "No guilds available." }
            }
            ChannelList {
                token: token.clone(),
                channels: channels.clone(),
                show_channel_pane: show_channel_pane.clone()
            }
        }
    }
}

#[component]
fn ChannelList(token: Signal<String>, channels: Signal<Option<Value>>, show_channel_pane: Signal<bool>) -> Element {
    let channels_array = channels()?.as_array().unwrap_or(&vec![]).clone();

    rsx! {
        div {
            class: {
                format_args!("channel-list-pane {}", if show_channel_pane() { "show" } else { "" })
            },
            h2 { "Channels" }
            if !channels()?.is_null() {
                ul {
                    for channel in channels_array {
                        li {
                            class: "channel-item",
                            button {
                                class: "channel-button",
                                onclick: move |_| {
                                    info!("Clicked channel: {}", channel["name"].as_str().unwrap_or("Unknown Channel"));
                                },
                                {channel["name"].as_str().unwrap_or("Unknown Channel")}
                            }
                        }
                    }
                }
            }
        }
    }
}
