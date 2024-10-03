use dioxus:: prelude::*;

// Api mongo structs
use futures::executor::block_on;
use dioxus_logger::tracing::{info, error, warn};
use mongodb::{sync::Client, bson::doc};
use crate::api::mongo_format::mongo_structs::*;
use crate::api::slack::event_server::*;
use crate::api::slack::{self, server_utils::*};

use futures_util::StreamExt;

use std::sync::{Arc};
use tokio::sync::Mutex;

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
    let user_lock_new = Arc::clone(&user_lock());
    let client_lock_new = Arc::clone(&client_lock());
    let user_lock_verify = Arc::clone(&user_lock());

    let handle_click = move |_|{
        let mut user = block_on(async{
            user_lock_verify.lock().await
        });

        println!("User: {:?}", user);
    };

    use_effect({
        tokio::spawn(async move {
            let _ = events_api(None, user_lock_new, client_lock_new).await;
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
            onclick: handle_click,
            "Install Workspace" 
        }
    }

}