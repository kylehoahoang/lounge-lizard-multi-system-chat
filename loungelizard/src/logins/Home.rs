use dioxus::prelude::*;
use dioxus_logger::tracing::warn;
use serde::de::value::EnumAccessDeserializer;
use crate::{AppRoute};
use futures::executor::block_on;

use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};
use crate::api::mongo_format::mongo_structs::*;
use crate::api::mongo_format::mongo_funcs::*;

use mongodb::{sync::Client, error::Result as MongoResult, bson::doc};
use dioxus_logger::tracing::{info, error, Level};

use std::sync::{Arc};
use tokio::sync::Mutex;
use regex::Regex;

#[component]
pub fn HomeLogin (confirmation: Signal<bool>) -> Element {

    // ! User Mutex Lock to access the user data
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    let client_lock = use_context::<Signal<Arc<Mutex<Option<Client>>>>>();
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

        let mongo_lock_copies = client_lock().clone();
        let user_lock_copies = user_lock().clone();

        let mongo_client = block_on(async {
            mongo_lock_copies.lock().await
        });

        let mut user = block_on(
            async{
                user_lock_copies.lock().await
            }
        );

         // Clone the client if it exists (since we can't return a reference directly)
        if let Some(client) = mongo_client.as_ref() {
            // Convert the function into async and spawn it on the current runtime
            let client_clone = client.clone();  // Clone the client to avoid ownership issues

            // Add personal info to USER structs before checking for duplicates
            user.username = username().clone();
            user.email = email().clone();
            user.password = password().clone();

            let user_clone = user.clone();
            
            // Use `tokio::spawn` to run the async block
            block_on(async move {
                let db = client_clone.database(MONGO_DATABASE);
                let user_collection = db.collection::<User>(MONGO_COLLECTION);
                
                match user_collection
                    .find_one(doc! { 
                        "$or": [{"username": &user_clone.username},
                                {"email": &user_clone.email}] })
                    .await {

                    Ok(Some(_)) => {
                       warn!("User already exists"); 
                    }
                    Ok(None) => {
                        info!("No match found, adding user");
                        match user_collection.insert_one(user_clone).await{
                            Ok(_) => {
                                info!("User added successfully");
                                confirmation.set(true);
                                logged_in.set(true);
                            }
                            Err(e) => {
                                error!("Something went wrong: {:#?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Something went wrong: {:#?}", e); 
                    }
                }
            });

        } else {
            warn!("MongoDB client not found in global state.");
        }
    
    };

    // ! Function to handle login
    let handle_login = move |_| {
        let mongo_lock_copies = client_lock().clone();
        let user_lock_copies = user_lock().clone();

        let mongo_client = block_on(async {
            mongo_lock_copies.lock().await
        });

        let mut user = block_on(async{
            user_lock_copies.lock().await
        }); 
    
         // Clone the client if it exists (since we can't return a reference directly)
        if let Some(client) = mongo_client.as_ref() {
            // Convert the function into async and spawn it on the current runtime
            let client_clone = client.clone();  // Clone the client to avoid ownership issues

            

            // Use `tokio::spawn` to run the async block
            block_on(async move {
                let db = client_clone.database("MultisystemChat");
                let user_collection = db.collection::<User>("LoungeLizard");
                // Lock user here for async access
                
                match user_collection
                    .find_one(doc! {
                        "$or": [
                            {"username": &username(), "password": &password()},
                            {"email": &username(), "password": &password()}
                        ]
                    })
                    .await {

                    Ok(Some(logged_user)) => {
                       info!("Logging in"); 
                       confirmation.set(true);
                       logged_in.set(true);
                       *user = logged_user;
                    }
                    Ok(None) => {
                        warn!("Passoword is incorrect");
                    }
                    Err(e) => {
                        error!("Something went wrong: {:#?}", e); 
                    }
                }
            });

        } else {
            warn!("MongoDB client not found in global state.");
        }
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