use dioxus::prelude::*;
use crate::Route;
use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};


#[component]
pub fn SlackLogin (show_slack_login_pane: Signal<bool>) -> Element {
    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());

    let mut logged_in = use_signal(|| false);
    let mut login_error = use_signal(|| None::<String>);

    let handle_login = move |_| {
        let username = username.clone();
        let password = password.clone();

    };
    let handle_new_user = move |_| {
        let ctx = ClipboardContext::new().unwrap();
        let token = ctx.get_text().unwrap_or("".to_string());
	    
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

             // Horizontal line
             div {
                style: "height: 2px; background-color: white; margin: 10px 10px; width: 100%;",
            }
            h1 {
                style: "color: white; font-size: 20px;",
                "New User Setup"
            }

            a {
                href: "https://api.slack.com/apps", // The URL you want to navigate to
                target: "_top",               // Opens in a new tab (optional)
                button { 
                    class: "login-button",
                    "Slack Api" 
                }
            }
       
            Link { 
                to: if true {Route::Slack {}} else {Route::Home {}},
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