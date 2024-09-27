use dioxus::prelude::*;

use crate::api::discord_api;
use dioxus_logger::tracing::{info};
use futures::executor::block_on;
use serde_json::Value;

#[component]
pub fn DiscordLogin(show_discord_login_pane: Signal<bool>, show_discord_server_pane: Signal<bool>, discord_token: Signal<String>, discord_guilds: Signal<Value>) -> Element {
    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());

    let mut login_error = use_signal(|| None::<String>);

    let handle_login = move |_| {
        let username = username.clone();
        let password = password.clone();

        block_on(async move {
            match discord_api::login_request(username.to_string(), password.to_string()).await {
                Ok((user_id, auth_discord_token)) => {
                    discord_token.set(auth_discord_token); // Call the success handler
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
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| {show_discord_login_pane.set(false) },
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

            if let Some(error) = login_error() {
                p { "Login failed: {error}" }
            }
        }
    }
}

