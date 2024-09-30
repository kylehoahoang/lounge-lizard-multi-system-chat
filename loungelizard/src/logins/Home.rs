use dioxus::prelude::*;
use crate::{AppRoute};

use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};
use crate::api::mongo_format::mongo_structs::*;

use std::sync::{Arc, Mutex};
use regex::Regex;

#[component]
pub fn HomeLogin (confirmation: Signal<bool>) -> Element {

    // ! User Mutex Lock to access the user data
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    // ! ========================= ! //

    // ? Place holders for returning users 
    let mut username = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());

    // ? Place holders for new users
    let mut email = use_signal(|| "".to_string());
    let mut retype_password = use_signal(|| "".to_string());
    let mut password_match = use_signal(|| false);
    let mut correct_password = use_signal(|| false);

    let mut logged_in = use_signal(|| false);
    let mut new_user_setup = use_signal(|| false); 
    let mut login_error = use_signal(|| None::<String>);

    // ! Helper function to check if all fields are filled in
    let is_valid = || {
        // Check if all fields are filled in
        !username().is_empty() && 
        !password().is_empty() && 
        !email().is_empty() &&
        password_match() &&
        correct_password() 
    };

    // ! Helper function to check if all fields are filled in for login
    let is_valid_login = || {
        // Check if all fields are filled in
        !username().is_empty() && 
        !password().is_empty() && 
        correct_password() 
    };

    // ! Function to handle new user signup
    let handle_new_user = move |_| {
       // ! Once Logged In

       confirmation.set(true);
       logged_in.set(true);
    
    };

    // ! Function to handle login
    let handle_login = move |_| {
        // ! Once Logged In 
        confirmation.set(true);
        logged_in.set(true);
        
    };

    // ! Function to toggle new user signup
    let toggle_new_user = 
        move |_| {new_user_setup.set(!new_user_setup());};

    rsx! {
        div {
            class: format_args!("discord-login {}",  if !logged_in() {"visible"} else {""}),
            if !new_user_setup() 
            {
                // ! Login Form
                input {
                    class: "login-input",
                    value: "{username}",
                    placeholder: "Username/Email",
                    oninput: move |event| username.set(event.value())
                }
                div {
                    class: "password-container", // Container for the password input and check mark
                    input {
                        class: "login-input",
                        r#type: "password",
                        value: "{password}",
                        placeholder: "Password: 8-16 Aa",
                        oninput: move |event| {
                            password.set(event.value());
                            correct_password.set(
                                is_valid_password(&event.value())
                            );
                            
                        }
                    }
                    // Check mark that appears conditionally
                    if correct_password() &&
                        password().len() > 0
                    {
                        img {
                            src: "assets/green-check.webp",
                            width: "25px",
                            height: "25px",
                        }
                    }
                    else if password().len() > 0
                    {
                        img {
                            src: "assets/red-x.webp",
                            width: "25px",
                            height: "25px",
                        }
                    }

                }
                div { 
                    class: "flex flex-row space-x-10", // Creates a horizontal layout with spacing
                    if is_valid_login() {
                        button { 
                            class: "login-button",
                            onclick: handle_login, 
                            "Login" 
                        }
                    } else {
                        // Disabled state
                        button { 
                            class: "login-button opacity-50 cursor-not-allowed",
                            "Fill in all fields"
                        }
                    }
                    button { 
                        class: "login-button",
                        onclick: toggle_new_user,
                        "New User" 
                    }
                }
            }
            else 
            {
                // ! New User Form
                input {
                    class: "login-input",
                    value: "{email}",
                    placeholder: "Email",
                    oninput: move |event| email.set(event.value())
                }
                input {
                    class: "login-input",
                    value: "{username}",
                    placeholder: "Username",
                    oninput: move |event| username.set(event.value())
                }

                div {
                    class: "password-container", // Container for the password input and check mark
                    input {
                        class: "login-input",
                        r#type: "password",
                        value: "{password}",
                        placeholder: "Password: 8-16 Aa",
                        oninput: move |event| {
                            password.set(event.value());
                            correct_password.set(
                                is_valid_password(&event.value())
                            );
                            
                        }
                    }
                    // Check mark that appears conditionally
                    if correct_password() &&
                        password().len() > 0
                    {
                        img {
                            src: "assets/green-check.webp",
                            width: "25px",
                            height: "25px",
                        }
                    }
                    else if password().len() > 0
                    {
                        img {
                            src: "assets/red-x.webp",
                            width: "25px",
                            height: "25px",
                        }
                    }

                }

                div {
                    class: "password-container", // Container for the password input and check mark
                    input {
                        class: "login-input",
                        r#type: "password",
                        value: "{retype_password}",
                        placeholder: "Retype Password",
                        oninput: move |event| {

                            retype_password.set(event.value());

                            if retype_password() != password() {
                               password_match.set(false);
                            } else {
                                password_match.set(true);
                            }
                        }
                    }
                    // Check mark that appears conditionally
                    if password_match() &&
                        retype_password().len() > 0
                    {
                        img {
                            src: "assets/green-check.webp",
                            width: "25px",
                            height: "25px",
                        }
                    }
                    else if retype_password().len() > 0
                    {
                        img {
                            src: "assets/red-x.webp",
                            width: "25px",
                            height: "25px",
                        }
                    }

                }
                div { 
                    class: "flex flex-row space-x-10", // Creates a horizontal layout with spacing
                    if is_valid() {
                        button { 
                            class: "login-button",
                            onclick: handle_new_user, 
                            "Create Account" 
                        }
                    } else {
                        // Disabled state
                        button { 
                            class: "login-button opacity-50 cursor-not-allowed",
                            "Fill in all fields"
                        }
                    }
                    button { 
                        class: "login-button",
                        onclick: toggle_new_user,
                        "Existing User" 
                    }
                } 
            
            }
            if let Some(error) = login_error() {
                p { "Login failed: {error}" }
            }
        
        }
    }
}
fn is_valid_password(password: &str) -> bool {
    // Regex for a password that is 8-16 characters long and contains only letters and numbers
    let re = Regex::new(r"^[a-zA-Z0-9]{8,16}$").unwrap();
    re.is_match(password)
}