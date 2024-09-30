use dioxus::prelude::*;

// Api mongo structs
use crate::api::mongo_format::mongo_structs::*;
use std::sync::{Arc, Mutex};

#[component]
pub fn Slack() -> Element {
    // ! User Mutex Lock to access the user data
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    //let client_lock = use_context::<Signal<Arc<Mutex<Option<Client>>>>>();
    // ! ========================= ! //
    
    if let Ok(mut user_lock) = user_lock().lock() {
        
        println!("Updated User Data: {:#?}", user_lock);
    } else {
        println!("Failed to acquire lock on the user.");
    }
   
    rsx! { h2 { "Blog Post:" } }
}