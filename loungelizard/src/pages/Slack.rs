use dioxus:: prelude::*;

// Api mongo structs
use futures::executor::block_on;
use dioxus_logger::tracing::{info, error, warn};
use mongodb::{sync::Client, bson::doc};
use crate::api::mongo_format::mongo_structs::*;
use crate::api::slack::event_server::*;
use crate::api::slack::{self, server_utils::*};
use std::sync::{Arc, Mutex};
use futures_util::StreamExt;

#[derive(Debug)]
struct UserStateExample(u64);

enum Action {
    Idle,
    Increment,
    Decrement,
    Reset,
}

#[component]
pub fn Slack() -> Element {
    // ! User Mutex Lock to access the user data
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    let client_lock = use_context::<Signal<Arc<Mutex<Option<Client>>>>>();

    // Lock the user context and clone it for async context
    let user = {
        let user_clone = user_lock().clone();
        let user_guard = user_clone.lock().unwrap();
        user_guard.clone()
    };

    use_effect({
        tokio::spawn(async move {
            let _ = events_api(None, &user).await;
        });

        // Return a cleanup function if needed (none in this case)
        || {}
    });

    // use_coroutine(|mut rx: UnboundedReceiver<Action>| { async move {
    //     let mut current_action = use_signal(|| Action::Increment); 

    //     while let Some(msg) = rx.next().await {
            
    //     }
    // }
    // });
    // ! ========================= ! //
    



    rsx! { 
        button { 
            class: "login-button",
            onclick: move |_| {
                println!("User: {:#?}", user_lock().clone().lock().unwrap().slack);
            },
            "Install Workspace" 
        }
    }

}