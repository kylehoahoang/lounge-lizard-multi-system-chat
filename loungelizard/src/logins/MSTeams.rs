use dioxus::prelude::*;
use crate::api::mongo_format::mongo_structs::*;

use std::sync::{Arc, Mutex};


#[component]
pub fn MSTeamsLogin (show_teams_login_pane: Signal<bool>) -> Element {

   // ! User Mutex Lock to access the user data
   let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
   // ! ========================= ! //

    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());

    let mut logged_in = use_signal(|| false);
    let mut login_error = use_signal(|| None::<String>);

    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());

    let mut new_user = use_signal(|| false);

    let mut login_error = use_signal(|| None::<String>);

    let handle_login = move |_| {
        let username = username.clone();
        let password = password.clone();

    };

    rsx! {
        div {
            class: format_args!("teams-login {}", if show_teams_login_pane() { "visible" } else { "" }),
            img {
                src: "assets/msteams_logo.png",
                alt: "MSTeams Logo",
                width: "50px",
                height: "50px",
            }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| { show_teams_login_pane.set(false) },
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
            button { 
                class: "New User",
                onclick: handle_login, "Login" 
            }

            if let Some(error) = login_error() {
                p { "Login failed: {error}" }
            }
        }
    }
}