#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, error, warn, Level};


use futures::executor::block_on;
use serde_json::Value;
use tokio::time;
use tokio::runtime::Runtime;
use futures_util::StreamExt;
use chrono::{DateTime, Utc, NaiveDateTime};

// * Front End Files
mod front_ends;
use front_ends::Slack::*;

// * Regular Page Routing Files 
mod pages;
use pages::Discord::*;
use pages::MSTeam::*;
use pages::Slack::*;
use pages::Home::*;

// * Login Page Routing Files
mod logins;

// * Api server files
mod api;
use api::discord::discord_api;
use api::mongo_format::mongo_structs::*;
use api::mongo_format::mongo_funcs::*; 

mod comp;

use lazy_static::lazy_static;
use std::sync::{Arc};
use tokio::sync::Mutex;


use mongodb::{sync::Client, bson::doc};


#[derive(Clone, Routable, Debug, PartialEq)]
enum AppRoute {
    #[route("/")]
    Home {}
}

// Global User instance using lazy_static
lazy_static! {
    static ref GLOBAL_USER: Arc<Mutex<User>> = Arc::new(Mutex::new(User::default()));
    static ref GLOBAL_MONGO_CLIENT: Arc<Mutex<Option<Client>>> = Arc::new(Mutex::new(None));
}

fn main() {

    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting application...");

    // Call init_mongo_client and set the result in GLOBAL_MONGO_CLIENT
    let client_result: Result<Option<Client>, mongodb::error::Error> = init_mongo_client(); 

    match client_result {
        Ok(Some(client)) => {
            let mut global_client = block_on(async {
                GLOBAL_MONGO_CLIENT.lock().await
            });
            //let mut global_client = GLOBAL_MONGO_CLIENT.lock().unwrap(); // Lock the mutex
            *global_client = Some(client); // Update the client inside the mutex
            info!("MongoDB client set successfully in global state.");
        }
        Ok(None) => {
            warn!("Failed to initialize MongoDB client.");
        }
        Err(e) => {
            error!("Unexpected error while initializing MongoDB client: {:?}", e);
        }
    }

    let cfg = dioxus::desktop::Config::new()
        .with_custom_head(r#"<link rel="stylesheet" href="tailwind.css">"#.to_string());

    LaunchBuilder::desktop()
        .with_cfg(cfg)
        .launch(App);

}

#[component]
fn App() -> Element {

    // Create a global signal for the Arc<Mutex<User>> data
    let user_lock = use_signal(|| GLOBAL_USER.clone());
    let client_lock = use_signal(|| GLOBAL_MONGO_CLIENT.clone());

    provide_context(user_lock.clone());
    provide_context(client_lock.clone());

    rsx! { Router::<AppRoute> {} }
}