use dioxus::prelude::*;

// Api mongo structs
use crate::api::mongo_format::mongo_structs::*;
use std::sync::{Arc};
use tokio::sync::Mutex;

#[component]
pub fn MSTeams() -> Element {
   // ! User Mutex Lock to access the user data
   let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
   // ! ========================= ! //
   
    rsx! { h2 { "Blog Post:" } }
}