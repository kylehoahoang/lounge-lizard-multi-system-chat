use dioxus::prelude::*;
use crate::{AppRoute};

use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};
use crate::api::mongo_format::mongo_structs::*;

use std::sync::{Arc, Mutex};

#[component]
pub fn SlackLogin (show_slack_login_pane: Signal<bool>) -> Element {

    // ! User Mutex Lock to access the user data
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    // ! ========================= ! //

    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());

    let mut logged_in = use_signal(|| false);
    let mut login_error = use_signal(|| None::<String>);

    let handle_new_user = move |_| {
        // let ctx = ClipboardContext::new().unwrap();
        // let token = ctx.get_text().unwrap_or("".to_string());
    
        if let Ok(mut user_lock) = user_lock().lock() {
            let new_username = "NewUsername".to_string();
            
            // Modify the `User` struct directly.
            user_lock.username = new_username;

            println!("Updated User Data: {:#?}", user_lock);
        } else {
            println!("Failed to acquire lock on the user.");
        }
    };

    rsx! {
        div {
            class: format_args!("slack-login {}", if show_slack_login_pane() { "visible" } else { "" }),
            img {
                src: "assets/slack_logo.png",
                alt: "Slack Logo",
                width: "50px",
                height: "50px",
            }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| {show_slack_login_pane.set(false)},
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
           
            a {
                href: "https://api.slack.com/apps", // The URL you want to navigate to
                target: "_top",               // Opens in a new tab (optional)
                button { 
                    class: "login-button",
                    "Slack Api" 
                }
            }
            button { 
                class: "login-button",
                onclick: handle_new_user, "Add WorkSpace" 
            }
            Link { 
                to: if true {AppRoute::Slack {}} else {AppRoute::Home {}},
                button { 
                    class: "login-button",
                    onclick: handle_new_user, "Add WorkSpace" 
                }
            }

            if let Some(error) = login_error() {
                p { "Login failed: {error}" }
            }
        }
    }
}

fn check_token(token: &str) -> bool {
    true
}