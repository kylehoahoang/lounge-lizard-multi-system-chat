#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};

// * Login Page Routing Files
use crate::logins::Discord::* ;
use crate::logins::Slack::* ;
use crate::logins::MSTeams::* ;

// Api mongo structs
use crate::api::mongo_format::mongo_structs::*;

#[component]
pub fn Home() -> Element {
   // ! User Mutex Lock to access the user data
   let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
   // ! ========================= ! //
    
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


    let handle_discord_click = move |_| {
        if discord_token.to_string() == ""{
            show_discord_login_pane.set(!show_discord_login_pane());

            // Set Other tokens to false
            show_slack_login_pane.set(false);
            show_teams_login_pane.set(false);
        }
        else {
            show_discord_server_pane.set(!show_discord_server_pane());
        }
    };

    let handle_slack_click = move |_| {
        if slack_token.to_string() == "" {
            show_slack_login_pane.set(!show_slack_login_pane());

            // Set Other tokens to false
            show_discord_login_pane.set(false);
            show_teams_login_pane.set(false);
            
        }
        else {

        }
       
        
    };

    let handle_teams_click = move |_| {
        if teams_token.to_string() == "" {
            show_teams_login_pane.set(!show_teams_login_pane());

            // Set Other tokens to false
            show_discord_login_pane.set(false);
            show_slack_login_pane.set(false);
        }
        else {

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
                        format_args!("white-square {}", if show_discord_login_pane() || show_discord_server_pane() { "opaque" } else { "transparent" })
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
                        format_args!("white-square {}", if show_slack_login_pane() { "opaque" } else { "transparent" })
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
                        format_args!("white-square {}", if show_teams_login_pane() { "opaque" } else { "transparent" })
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

                h1 { 
                    class: "welcome-message", 
                    "welcome back to lounge lizard" 
                }
                // Sliding login pane
                div {
                    class: {
                        format_args!("login-pane {}",
                            if show_discord_login_pane() || show_slack_login_pane() || show_teams_login_pane(){ "show" } else { "" })
                    },
                    if show_discord_login_pane() {
                        DiscordLogin { 
                            show_discord_login_pane: show_discord_login_pane.clone(),
                            show_discord_server_pane: show_discord_server_pane.clone(), 
                            discord_token: discord_token.clone(),
                            discord_guilds: discord_guilds.clone(),
                            //user
                        }
                    }
                    else if show_slack_login_pane() {
                        SlackLogin { 
                            show_slack_login_pane: show_slack_login_pane.clone(),
                            //user
                        }
                    }
                    else if show_teams_login_pane() {
                        MSTeamsLogin { 
                            show_teams_login_pane: show_teams_login_pane.clone(),
                           //user
                        }
                    }
                    
                }

                // // Bottom pane for servers
                // DiscordBottomPane { 
                //     show_discord_server_pane: show_discord_server_pane.clone(),
                //     discord_guilds: discord_guilds.clone(),
                //     discord_token: discord_token.clone()
                // }, 
            }
        }
    }
}
