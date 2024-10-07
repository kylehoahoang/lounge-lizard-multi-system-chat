#![allow(non_snake_case)]

use dioxus::prelude::*;
use slack_morphism::prelude::*;
use serde_json::Value;
use std::sync::{Arc};
use tokio::sync::Mutex;
use futures::executor::block_on;
use std::{fs, result};
use url::Url;
use crate::api::slack::ngrok_s::*;
use crate::api::slack::server_utils::setup_server::*;
use dioxus_logger::tracing::{info, error, warn};
use crate::{AppRoute};


// * Login Page Routing Files
use crate::logins::Discord::* ;
use crate::logins::Slack::* ;
use crate::logins::MSTeams::* ;
use crate::logins::Home::* ;

use crate::pages::Discord::Discord as Discord_p;
use crate::pages::Slack::Slack as Slack_p;
use crate::pages::MSTeam::MSTeams as MSTeams_p;

// Api mongo structs
use crate::api::mongo_format::mongo_structs::*;

#[component]
pub fn Home() -> Element {
   // ! User Mutex Lock to access the user data
   let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
   // ! ========================= ! //

   let user_lock_clone_slack = Arc::clone(&user_lock());
   let user_lock_clone_discord = Arc::clone(&user_lock());
   let user_lock_clone_teams = Arc::clone(&user_lock());
    
    // Discord Values 
    let mut show_discord_login_pane = use_signal(|| false);
    let mut show_discord_server_pane = use_signal(|| false);
    let mut discord_token = use_signal(|| "".to_string());
    let mut discord_guilds = use_signal(|| Value::Null);


    // Slack Values
    let mut show_slack_login_pane = use_signal(|| false);
    let mut slack_token = use_signal(|| "".to_string());

    // MSTeams Values
    let mut show_teams_login_pane = use_signal(|| false);
    let mut teams_token = use_signal(|| "".to_string());

    let mut logged_in = use_signal(|| false);

    let mut current_platform = use_signal(|| "None".to_string());


    let handle_discord_click = move |_| {

        let user = block_on(async {
            user_lock_clone_discord.lock().await
        });

        if user.discord.token == ""{
            show_discord_login_pane.set(!show_discord_login_pane());

            // Set Other tokens to false
            show_slack_login_pane.set(false);
            show_teams_login_pane.set(false);
        }
        else {
            show_discord_server_pane.set(!show_discord_server_pane());
            current_platform.set("Discord".to_string());
        }
    };

    let handle_slack_click = move |_| {

        let user = block_on(async {
            user_lock_clone_slack.lock().await
        });

        // Check if the app id has been set, this means the user has logged in
        if user.slack.app_id == "" {
            show_slack_login_pane.set(!show_slack_login_pane());

            // Set Other tokens to false
            show_discord_login_pane.set(false);
            show_teams_login_pane.set(false);
            
        }
        else if current_platform().to_string() != "Slack" {

            let result = block_on( async {
                update_slack_app(user.clone()).await
            });

            match result {
                Ok(_) => {
                    current_platform.set("Slack".to_string());
                },
                Err(_) => {
                    error!("Failed to Initialize Ngrok Server");
                }
            }
        }
       
        
    };

    let handle_teams_click = move |_| {

        let user = block_on(async {
            user_lock_clone_teams.lock().await
        });

        if user.ms_teams.token == "" {
            show_teams_login_pane.set(!show_teams_login_pane());

            // Set Other tokens to false
            show_discord_login_pane.set(false);
            show_slack_login_pane.set(false);
        }
        else {
            current_platform.set("MSTeams".to_string());
        }
       
        
    };


    rsx! {
        div {
            class: "main-container",

            // Left vertical bar
            div {
                class: "vertical-bar",
                div {
                    class: {
                        format_args!("white-square {}",
                            if (show_discord_login_pane() || show_discord_server_pane()) && logged_in()
                            { "opaque" } 
                            else 
                            { "transparent" })
                    },
                    img {
                        src: "assets/discord_logo.png",
                        alt: "Discord Logo",
                        width: "50px",
                        height: "50px",
                        style: "cursor: pointer;",
                        onclick: handle_discord_click,
                    }
                },
                div {
                    class: {
                        format_args!("white-square {}", 
                            if show_slack_login_pane() && logged_in() 
                            { "opaque" } 
                            else 
                            { "transparent" })
                    },
                    img {
                        src: "assets/slack_logo.png",
                        alt: "Slack Logo",
                        width: "50px",
                        height: "50px",
                        style: "cursor: pointer;",
                        onclick: handle_slack_click,
                    }
                },

                div {
                    class: {
                        format_args!("white-square {}",
                            if show_teams_login_pane() && logged_in() 
                            { "opaque" } 
                            else 
                            { "transparent" })
                    },
                    img {
                        src: "assets/msteams_logo.png",
                        alt: "MSTeams Logo",
                        width: "50px",
                        height: "50px",
                        style: "cursor: pointer;",
                        onclick: handle_teams_click,
                    }
                }
            }

            // Main content area
            div {
                class: "main-content",
//
            if !logged_in() {
                h1 { 
                    class: "welcome-message", 
                    "Welcome to Loung Lizard Chat"
                }
                
                HomeLogin {
                    confirmation: logged_in.clone()
                }
            }
            else 
            {

                if current_platform().to_string() == "Discord" {
                    Discord_p{}
                }
                else if current_platform().to_string() == "MSTeams" {
                    MSTeams_p{}
                }
                else if current_platform().to_string() == "Slack" {
                    Slack_p{current_platform: current_platform.clone()}
                }
                else{
                    h2 { 
                        class: "welcome-message", 
                        "Sign in to the platforms and start chatting!"
                    }
                }
                
            }
    

// Sliding login pane
                // Sliding login pane
                
                div {
                    class: {
                        format_args!("login-pane {}",
                            if (show_discord_login_pane() || show_slack_login_pane() || show_teams_login_pane()) && logged_in()
                            { "show" } else { "" })
                    },
                    if show_discord_login_pane() {
                        DiscordLogin { 
                            show_discord_login_pane: show_discord_login_pane.clone(),
                            show_discord_server_pane: show_discord_server_pane.clone(), 
                            discord_token: discord_token.clone(),
                            discord_guilds: discord_guilds.clone(),
                            current_platform: current_platform.clone(),
                           
                        }
                    }
                    else if show_slack_login_pane() {
                        SlackLogin { 
                            show_slack_login_pane: show_slack_login_pane.clone(),
                            current_platform: current_platform.clone(),
                        }
                    }
                    else if show_teams_login_pane() {
                        MSTeamsLogin { 
                            show_teams_login_pane: show_teams_login_pane.clone(),
                            current_platform: current_platform.clone(),
                        }
                    }
                    
                }
            }
        }
    }
}
