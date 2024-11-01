use dioxus::prelude::*;
use serde_json::{json, Value};
use tracing::info;
use futures::executor::block_on;
use chrono::{DateTime, Utc};
use crate::api::ms_teams::ms_teams_api::*;

// Api mongo structs
use crate::api::mongo_format::mongo_structs::*;
use std::sync::Arc;
use tokio::sync::Mutex;

#[component]
pub fn MSTeams(show_teams_server_pane: Signal<bool>, teams_list: Signal<Value>) -> Element {
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    let user_teams = Arc::clone(&user_lock());

    let current_team_id = use_signal(|| None::<Value>);
    block_on(async move {
        let ms_teams_token = user_teams.lock().await.ms_teams.token.clone();

        match get_teams(&ms_teams_token).await {
            Ok(teams_response) => {
                //println!("{}", teams_response);
                teams_list.set(teams_response.clone());
                info!("Teams list retrieval successful");
            }
            Err(e) => {
                info!("Teams list retrieval failed: {}", e);
            }
        }
    });
    rsx! {
        TeamsBottomPane{
            show_teams_server_pane: show_teams_server_pane.clone(),
            teams_list: teams_list.clone(),
            user: user_lock,
            current_team_id: current_team_id.clone()
        },
    }
}

#[component]
fn TeamsBottomPane(show_teams_server_pane: Signal<bool>, teams_list: Signal<Value>, user: Signal<Arc<Mutex<User>>>, current_team_id: Signal<Option<Value>>) -> Element {
    let teams_array = teams_list()
        .get("value")
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![])
        .clone();
    let mut channels = use_signal(|| None::<Value>);
    let mut fetch_error = use_signal(|| None::<String>);
    let mut show_channel_pane = use_signal(|| false);


    let mut handle_get_channels = move |team: Value, user_lock_api: Arc<Mutex<User>>| {
        current_team_id.set(Some(team.clone()));
        block_on(async move {
            let user_lock_api = user_lock_api.lock().await;
            let ms_teams_token = user_lock_api.ms_teams.token.clone();
            let team_id = team.get("id").and_then(|v| v.as_str()).unwrap_or("");

            match get_channels(&ms_teams_token, team_id).await {
                Ok(channels_data) => {
                    channels.set(Some(channels_data));
                    show_channel_pane.set(true);
                }
                Err(e) => {
                    fetch_error.set(Some(e.to_string()));
                }
            }
        });
    };

    rsx! {
        div {
            class: {
                format_args!("teams-bottom-pane {}", if show_teams_server_pane() { "show" } else { "" })
            },
            h2 { "MS Teams" }
            if !teams_list().is_null() {
                ul {
                    class: "team-list",
                    for team in teams_array {
                        li {
                            class: "team-item",
                            button {
                                class: "team-button",
                                onclick: move |_| {
                                    handle_get_channels(team.clone(), Arc::clone(&user()))
                                },
                                {team.get("displayName").and_then(|v| v.as_str()).unwrap_or("Unknown ID").to_string()}
                            }
                        }
                    }
                }
            }
            else {
                p { "No teams available." }
            }
            ChannelList {
                user: user.clone(),
                channels: channels.clone(),
                show_channel_pane: show_channel_pane.clone(),
                show_teams_server_pane: show_teams_server_pane.clone(),
                current_team_id: current_team_id.clone()
            }
        }
    }
}

#[component]
fn ChannelList(user: Signal<Arc<Mutex<User>>>, channels: Signal<Option<Value>>, show_channel_pane: Signal<bool>, show_teams_server_pane: Signal<bool>, current_team_id: Signal<Option<Value>>) -> Element {
    let channels_array = channels()
        .as_ref()
        .and_then(|c| c.get("value"))
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![])
        .clone();
    let mut messages = use_signal(|| None::<Value>);
    let mut fetch_error = use_signal(|| None::<String>);
    let mut show_channel_messages_pane = use_signal(|| false);
    let mut current_channel_id = use_signal(|| None::<Value>);

    let handle_get_channel_messages = move |channel: Value, user_lock_api: Arc<Mutex<User>>| {
        current_team_id.with(|team_opt| {
            if let Some(team) = team_opt {
                let team_id = team.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let channel_id = channel.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let channel_name = channel.get("displayName").and_then(|v| v.as_str()).unwrap_or("");

                let channel_id_clone = channel_id.to_string();
                let channel_name_clone = channel_name.to_string();
     
                block_on(async move{
                    let user_lock_api = user_lock_api.lock().await;
                    let ms_teams_token = user_lock_api.ms_teams.token.clone();
                    match get_messages(&ms_teams_token, team_id, channel_id).await {
                        Ok(messages_data) => {
                            messages.set(Some(messages_data));
                            current_channel_id.set(Some(json!({
                                "id": channel_id_clone,
                                "name": channel_name_clone
                            })));
                            show_channel_messages_pane.set(true);
                        }
                        Err(e) => {
                            fetch_error.set(Some(e.to_string()));
                        }
                    }
                });
            }
        });
    };

    rsx! {
        div {
            class: {
                format_args!("channel-list-pane {}", if show_channel_pane() && show_teams_server_pane() { "show" } else { "" })
            },
            h2 { "Channels" }
            button {
                style: "position: absolute; top: 10px;right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move|_| { show_channel_pane.set(false);},
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    view_box: "0 0 24 24",
                    width: "30",
                    height: "30",
                    path {
                        d: "M18 6 L6 18 M6 6 L18 18",
                        fill: "none",
                        stroke: "f5f5f5",
                        stroke_width: "2"
                    }
                }
            }
            if !channels()?.is_null() {
                ul {
                    class: "channel-list",
                    for channel in channels_array {
                        li {
                            class: "channel-item",
                            button {
                                class: "channel-button",
                                onclick: move |_| {
                                    handle_get_channel_messages(channel.clone(), Arc::clone(&user()))
                                },
                                {channel.get("displayName").and_then(|v| v.as_str()).unwrap_or("Unknown ID").to_string()}
                            }
                        }
                    }
                }
            }
            ChannelMessages {
                user: user.clone(),
                messages: messages.clone(),
                show_channel_messages_pane: show_channel_messages_pane.clone(),
                current_team_id: current_team_id.clone(),
                current_channel_id: current_channel_id.clone(),
                show_teams_server_pane: show_teams_server_pane.clone()
            }
        }
    }
}

#[derive(Debug, Clone)]
struct EmptyStruct {} // Empty struct to use for coroutines (when you don't need to send anything into the coroutine)

#[component]
fn ChannelMessages (
    user: Signal<Arc<Mutex<User>>>,
    messages: Signal<Option<Value>>,
    show_channel_messages_pane: Signal<bool>,
    current_team_id: Signal<Option<Value>>,
    current_channel_id: Signal<Option<Value>>,
    show_teams_server_pane: Signal<bool>
) -> Element {
    let mut send_error = use_signal(|| None::<String>);
    let mut message_input = use_signal(|| "".to_string());

    let messages_data = messages()
        .as_ref()
        .and_then(|messages_list| messages_list.get("value"))
        .and_then(|v| v.as_array())
        .unwrap_or(&vec![])
        .clone();
    let user_lock_api = Arc::clone(&user());

    let handle_send_message = move |user_lock_api: Arc<Mutex<User>>| {
        if let Some(team) = current_team_id() {
            if let Some(channel) = current_channel_id() {
                let team_id = team.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let channel_id = channel.get("id").and_then(|v| v.as_str()).unwrap_or("");

                block_on(async move {
                    let user_lock_api = user_lock_api.lock().await;
                    let ms_teams_token = user_lock_api.ms_teams.token.clone();
                    match send_message(&ms_teams_token, team_id, channel_id, &message_input()).await {
                        Ok(_) => {
                            info!("Message sent successfully");
                            message_input.set("".to_string());
                        }
                        Err(e) => {
                            send_error.set(Some(e.to_string()));
                            info!("Message send failed: {}", e);
                        }
                    }
                    match get_messages(&ms_teams_token, team_id, channel_id).await {
                        Ok(updated_messages) => {
                            println!("{:?}", updated_messages);
                            messages.set(Some(updated_messages));
                            info!("Messages update successful");
                        }
                        Err(e) => {
                            send_error.set(Some(e.to_string()));
                            info!("Messages update failed: {}", e);
                        }
                    }
                });
            }
        }
    };

    let _fetch_messages = use_coroutine::<EmptyStruct, _, _>(|_rx| {
        let current_team_id = current_team_id.clone();
        let current_channel_id = current_channel_id.clone();
        let mut messages = messages.clone();
        async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                if let Some(team) = current_team_id() {
                    if let Some(channel) = current_channel_id() {
                        let team_id = team.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let channel_id = channel.get("id").and_then(|v| v.as_str()).unwrap_or("");

                        let user_lock_api = user_lock_api.lock().await;
                        let ms_teams_token = user_lock_api.ms_teams.token.clone();
                        match get_messages(&ms_teams_token, team_id, channel_id).await {
                            Ok(updated_messages) => {
                                messages.set(Some(updated_messages));
                            }
                            Err(e) => {
                                info!("Failed to fetch updated messages: {}", e);
                            }
                        }
                    }
                }
            }
        }
    });

    rsx! {
        div {
            class: {
                format_args!("channel-messages-list-pane {}", if show_channel_messages_pane() && show_teams_server_pane() { "show" } else { "" })
            },
            h2 { "Messages" }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| { show_channel_messages_pane.set(false); },
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    view_box: "0 0 24 24",
                    width: "30",
                    height: "30",
                    path {
                        d: "M18 6 L6 18 M6 6 L18 18",
                        fill: "none",
                        stroke: "#f5f5f5",
                        stroke_width: "2"
                    }
                }
            }
            ul {
                class: "messages-list",
                for message in messages_data {
                    li {
                        class: "messages-item",
                        div {
                            class: "message-header",
                            span {
                                class: "message-username",
                                {message.get("from").and_then(|m| m.get("user")).and_then(|u| u.get("displayName")).and_then(|v| v.as_str()).unwrap_or("Unknown User")}
                            }
                            span {
                                class: "message-date",
                                {format_timestamp(message.get("createdDateTime").and_then(|v| v.as_str()).unwrap_or(""))}
                            }
                        }
                        div {
                            class: "message-content",
                            {message.get("body").and_then(|b| b.get("content")).and_then(|v| v.as_str()).unwrap_or("Failed to display message.")}
                        }
                    }
                }
            }
            div {
                class: "message-input-container",
                input {
                    class: "message-input-box",
                    value: "{message_input}",
                    placeholder: "Enter your message.",
                    oninput: move |event| message_input.set(event.value())
                }
                button {
                    class: "send-button",
                    onclick: move |_| handle_send_message(Arc::clone(&user())),
                    "Send"
                }
            }
        }
    }
}

fn format_timestamp(timestamp: &str) -> String {
    // Parse the timestamp string into a DateTime object
    let parsed_timestamp = DateTime::parse_from_rfc3339(timestamp).unwrap_or_else(|_| Utc::now().into());
    
    // Format the date into a readable format, e.g., "Sep 26, 2024 12:45 PM"
    parsed_timestamp.format("%b %d, %Y %I:%M %p").to_string()
}