use dioxus::prelude::*;

// Api mongo structs
use crate::api::mongo_format::mongo_structs::*;
use std::sync::{Arc, Mutex};
use std::sync::Mutex as StdMutex;
#[component]
pub fn Discord() -> Element {
   // ! User Mutex Lock to access the user data
   let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
   // ! ========================= ! //
   
    rsx! { h2 { "Blog Post:" } }
}