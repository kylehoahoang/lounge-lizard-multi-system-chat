use dioxus::prelude::*;
use serde_json::Value;
use tracing::info;
use futures::executor::block_on;
use chrono::{DateTime, Local, Utc};
use std::collections::HashMap;
use crate::api::ms_teams::ms_teams_api::*;

// Api mongo structs
use crate::api::mongo_format::mongo_structs::*;
use std::sync::Arc;
use tokio::sync::Mutex;

type UserCache = HashMap<String, (String, String)>; // user_id -> displayName, profilePicture

#[component]
pub fn MSTeams(show_teams_server_pane: Signal<bool>) -> Element {
    
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();

    let selected_team_id = use_signal(|| None::<Value>);
    let selected_channel_id = use_signal(|| None::<Value>);
    let selected_user_id = use_signal(|| None::<Value>);

    let teams_list = use_signal(|| Value::Null);
    let channels_list = use_signal(|| Value::Null);
    let messages_list = use_signal(|| None::<Value>);
    let users_list = use_signal(|| UserCache::new());

    spawn(async move {
        let user_lock = Arc::clone(&user_lock());
        let mut teams_list = teams_list.clone();
        let mut selected_user_id = selected_user_id.clone(); 

        let user_lock_api = user_lock.lock().await;
        let ms_teams_token = user_lock_api.ms_teams.access_token.clone();

        match get_user(&ms_teams_token).await {
            Ok(user_data) => {
                selected_user_id.set(Some(user_data.clone()));
            }
            Err(e) => {
                eprintln!("Failed to retrieve user: {}", e);
            }
        }

        match get_teams(&ms_teams_token).await {
            Ok(teams_data) => {
                teams_list.set(teams_data.clone());
                info!("Teams list retrieval successful");
            }
            Err(e) => {
                eprintln!("Failed to retrieve teams: {}", e);
            }
        }
    });

    rsx! {
        div {
            class: "ms-teams-main-container",
            LeftSidebar {
                teams_list: teams_list.clone(),
                messages_list: messages_list.clone(),
                selected_team_id: selected_team_id.clone(),
                selected_channel_id: selected_channel_id.clone(),
                channels_list: channels_list.clone(),
                users_list: users_list.clone()
            },
            MiddlePanel {
                channels_list: channels_list.clone(),
                messages_list: messages_list.clone(),
                selected_team_id: selected_team_id.clone(),
                selected_channel_id: selected_channel_id.clone(),
                users_list: users_list.clone()

            },
            RightPanel {
                messages_list: messages_list.clone(),
                selected_team_id: selected_team_id.clone(),
                selected_channel_id: selected_channel_id.clone(),
                selected_user_id: selected_user_id.clone(),
                users_list: users_list.clone()
            }
        }
    }
}

#[component]
fn LeftSidebar(teams_list: Signal<Value>, messages_list: Signal<Option<Value>>, selected_team_id: Signal<Option<Value>>, selected_channel_id: Signal<Option<Value>>, channels_list: Signal<Value>, users_list: Signal<UserCache>) -> Element {
    
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    let teams_array = teams_list().as_array().unwrap_or(&Vec::new()).clone();
    let mut fetch_error = use_signal(|| None::<String>);
    let mut is_loading = use_signal(|| false);

    let mut handle_get_channels = move |user_lock_api: Arc<Mutex<User>>| {
        let team_id = selected_team_id().as_ref().and_then(|team| team.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        block_on(async move {
            let user_lock_api = user_lock_api.lock().await;
            let ms_teams_token = user_lock_api.ms_teams.access_token.clone();

            match get_users(&ms_teams_token, &team_id).await {
                Ok(users_data) => {
                    users_list.set(users_data);
                }
                Err(e) => {
                    fetch_error.set(Some(e.to_string()));
                }
            }

            match get_channels(&ms_teams_token, &team_id).await {
                Ok(channels_data) => {
                    channels_list.set(channels_data.clone());
                    if let Some(first_channel) = channels_data.get(0) {
                        selected_channel_id.set(Some(first_channel.clone()));
                        let channel_id = first_channel.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        match get_messages(&ms_teams_token, &team_id, &channel_id, &users_list()).await {
                            Ok(messages_data) => {
                                messages_list.set(Some(messages_data));
                            }
                            Err(e) => {
                                fetch_error.set(Some(e.to_string()));
                            }
                        }
                    }
                }
                Err(e) => {
                    fetch_error.set(Some(e.to_string()));
                }
            }
        });
        is_loading.set(false);
    };

    rsx! {
        div {
            class: "ms-teams-left-sidebar",
            for team in teams_array {
                if let team_copy = team.clone() {
                    button {
                        class: {
                            format!("ms-teams-team-icon {}",
                            if selected_team_id().as_ref().and_then(|v| v.get("id")).and_then(|v| v.as_str()) == team_copy.get("id").and_then(|v| v.as_str()) { "active" } else { "" })
                        },
                        disabled: is_loading(),
                        onclick: move |_| {
                            is_loading.set(true);
                            selected_team_id.set(Some(team.clone()));
                            handle_get_channels(Arc::clone(&user_lock()));
                        },
                        img {
                            src: team_copy.get("teamPicture").and_then(|v| v.as_str()).unwrap_or("https://via.placeholder.com/150"),
                            alt: team_copy.get("displayName").and_then(|v| v.as_str()).unwrap_or("Unknown ID").to_string(),
                        }
                        {team.get("displayName").and_then(|v| v.as_str()).unwrap_or("Unknown ID").to_string()},
                    }
                }
            }
        }
    }
}

#[component]
fn MiddlePanel(
    channels_list: Signal<Value>,
    messages_list: Signal<Option<Value>>,
    selected_team_id: Signal<Option<Value>>,
    selected_channel_id: Signal<Option<Value>>,
    users_list: Signal<UserCache>,
) -> Element {
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    let channels_array = channels_list().as_array().unwrap_or(&Vec::new()).clone();
    let mut fetch_error = use_signal(|| None::<String>);
    let mut is_loading = use_signal(|| false);

    let mut handle_get_channel_messages = move |user_lock_api: Arc<Mutex<User>>| {
        let team_id = selected_team_id().as_ref().and_then(|team| team.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let channel_id = selected_channel_id().as_ref().and_then(|team| team.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();

        spawn(async move {
            let user_lock_api = user_lock_api.lock().await;
            let ms_teams_token = user_lock_api.ms_teams.access_token.clone();

            match get_messages(&ms_teams_token, &team_id, &channel_id, &users_list()).await {
                Ok(messages_data) => {
                    messages_list.set(Some(messages_data));
                }
                Err(e) => {
                    fetch_error.set(Some(e.to_string()));
                }
            }
        });
        is_loading.set(false);
    };

    rsx! {
        div {
            class: "ms-teams-middle-panel",
            h2 { "Channels" }
            ul {
                class: "ms-teams-channel-list",
                for channel in channels_array {
                    button {
                        class: {
                            format!("ms-teams-channel-item {}",
                            if Some(channel.clone()) == selected_channel_id() { "active" } else { "" })
                        },
                        disabled: is_loading(),
                        onclick: move |_| {
                            is_loading.set(true);
                            selected_channel_id.set(Some(channel.clone()));
                            handle_get_channel_messages(Arc::clone(&user_lock()))
                        },
                        {channel.get("displayName").and_then(|v| v.as_str()).unwrap_or("Unknown ID").to_string()}
                    }
                }
            }
        }
    }
}

#[ignore = "irrefutable_let_patterns"]
#[component]
fn RightPanel(messages_list: Signal<Option<Value>>, selected_team_id: Signal<Option<Value>>, selected_channel_id: Signal<Option<Value>>, selected_user_id: Signal<Option<Value>>, users_list: Signal<UserCache>) -> Element {
    let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
    
    let mut send_error = use_signal(|| None::<String>);
    let mut message_input = use_signal(|| "".to_string());
    let mut message_subject_input = use_signal(|| "".to_string());
    let mut reply_input = use_signal(|| "".to_string());
    
    let mut selected_message_id = use_signal(|| None::<Value>);

    let mut show_subject_input = use_signal(|| false);
    let mut show_reply_input = use_signal(|| false);
    let mut show_emoji_list = use_signal(|| false);
    let mut show_reaction_list = use_signal(|| false);
    let mut show_reply_emoji_list = use_signal(|| false);

    let mut is_loading = use_signal(|| false);

    let messages_array = messages_list().as_ref().and_then(|value| value.as_array()).unwrap_or(&Vec::new()).clone();

    use_effect(move || {
        let user_lock = Arc::clone(&user_lock());
        let mut messages_list = messages_list.clone();
        let selected_team_id = selected_team_id.clone();
        let selected_channel_id = selected_channel_id.clone();
        let users_list = users_list.clone();
        let mut send_error = send_error.clone();

        spawn(async move {
            loop {
                if !is_loading() {
                    if let (Some(team), Some(channel)) = (selected_team_id(), selected_channel_id()) {
                        let team_id = team.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let channel_id = channel.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        
                        let user_lock_api = user_lock.lock().await;
                        let access_token = user_lock_api.ms_teams.access_token.clone();
    
                        match get_messages(&access_token, &team_id, &channel_id, &users_list()).await {
                            Ok(messages_data) => {
                                messages_list.set(Some(messages_data));
                            }
                            Err(e) => {
                                send_error.set(Some(e.to_string()));
                            }
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        });
    });

    let mut handle_send_message = move |user_lock_api: Arc<Mutex<User>>| {
        if let Some(team) = selected_team_id() {
            if let Some(channel) = selected_channel_id() {
                let team_id = team.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let channel_id = channel.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

                spawn(async move {
                    let user_lock_api = user_lock_api.lock().await;
                    let access_token = user_lock_api.ms_teams.access_token.clone();
                    match send_message(&access_token, &team_id, &channel_id, &message_input(), &message_subject_input()).await {
                        Ok(_) => {
                            info!("Message sent successfully");
                            message_input.set("".to_string());
                            message_subject_input.set("".to_string());
                        }
                        Err(e) => {
                            send_error.set(Some(e.to_string()));
                            info!("Message send failed: {}", e);
                        }
                    }
                    match get_messages(&access_token, &team_id, &channel_id, &users_list()).await {
                        Ok(updated_messages) => {
                            messages_list.set(Some(updated_messages).clone());
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
        is_loading.set(false);
    };

    let mut handle_send_reply = move |user_lock_api: Arc<Mutex<User>>| {
        let team_id = selected_team_id().as_ref().and_then(|team| team.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let channel_id = selected_channel_id().as_ref().and_then(|channel| channel.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let message_id = selected_message_id().as_ref().and_then(|message| message.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();

        spawn(async move {
            let user_lock_api = user_lock_api.lock().await;
            let access_token = user_lock_api.ms_teams.access_token.clone();
            match send_message_reply(&access_token, &team_id, &channel_id, &message_id, &reply_input()).await {
                Ok(_) => {
                    info!("Reply sent successfully");
                    reply_input.set("".to_string());
                }
                Err(e) => {
                    send_error.set(Some(e.to_string()));
                    info!("Reply send failed: {}", e);
                }
            }
            match get_messages(&access_token, &team_id, &channel_id, &users_list()).await {
                Ok(updated_messages) => {
                    messages_list.set(Some(updated_messages).clone());
                    info!("Messages update successful");
                }
                Err(e) => {
                    send_error.set(Some(e.to_string()));
                    info!("Messages update failed: {}", e);
                }
            }
        });
        is_loading.set(false);
    };

    let mut handle_send_reaction = move |emoji: &str, user_lock_api: Arc<Mutex<User>>| {
        let team_id = selected_team_id().as_ref().and_then(|team| team.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let channel_id = selected_channel_id().as_ref().and_then(|channel| channel.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let message_id = selected_message_id().as_ref().and_then(|message| message.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();

        block_on(async move {
            let user_lock_api = user_lock_api.lock().await;
            let access_token = user_lock_api.ms_teams.access_token.clone();

            match send_reaction(&access_token, &team_id, &channel_id, &message_id, emoji).await {
                Ok(()) => {
                }
                Err(e) => {
                    send_error.set(Some(e.to_string()));
                }
            }
            match get_messages(&access_token, &team_id, &channel_id, &users_list()).await {
                Ok(updated_messages) => {
                    messages_list.set(Some(updated_messages).clone());
                    info!("Messages update successful");
                }
                Err(e) => {
                    send_error.set(Some(e.to_string()));
                    info!("Messages update failed: {}", e);
                }
            }
        });
        is_loading.set(false);
    };

    let mut handle_send_reaction_reply = move |emoji: &str, user_lock_api: Arc<Mutex<User>>| {
        let team_id = selected_team_id().as_ref().and_then(|team| team.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let channel_id = selected_channel_id().as_ref().and_then(|channel| channel.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let message_id = selected_message_id().as_ref().and_then(|message| message.get("replyFromId")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let reply_id = selected_message_id().as_ref().and_then(|reply| reply.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();

        block_on(async move {
            let user_lock_api = user_lock_api.lock().await;
            let access_token = user_lock_api.ms_teams.access_token.clone();

            match send_reaction_reply(&access_token, &team_id, &channel_id, &message_id, &reply_id, emoji).await {
                Ok(()) => {
                }
                Err(e) => {
                    send_error.set(Some(e.to_string()));
                }
            }
            match get_messages(&access_token, &team_id, &channel_id, &users_list()).await {
                Ok(updated_messages) => {
                    messages_list.set(Some(updated_messages).clone());
                    info!("Messages update successful");
                }
                Err(e) => {
                    send_error.set(Some(e.to_string()));
                    info!("Messages update failed: {}", e);
                }
            }
        });
        is_loading.set(false);
    };

    let mut handle_remove_reaction = move |user_lock_api: Arc<Mutex<User>>| {
        let team_id = selected_team_id().as_ref().and_then(|team| team.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let channel_id = selected_channel_id().as_ref().and_then(|channel| channel.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let message_id = selected_message_id().as_ref().and_then(|message| message.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let user_id = selected_user_id().as_ref().and_then(|id| id.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();

        if let Some(reactions) = selected_message_id().as_ref().and_then(|message| message.get("reactions")).and_then(|r| r.as_array()) {
            for reaction in reactions {
                let reaction_user_id = reaction.get("user").and_then(|user| user.get("id")).and_then(|id| id.as_str()).unwrap_or("").to_string();
                if reaction_user_id == user_id {
                    let emoji = reaction.get("emoji").and_then(|v| v.as_str()).unwrap_or("");
                    block_on(async move {
                        let user_lock_api = user_lock_api.lock().await;
                        let access_token = user_lock_api.ms_teams.access_token.clone();
                
                        match remove_reaction(&access_token, &team_id, &channel_id, &message_id, emoji).await {
                            Ok(()) => {
                            }
                            Err(e) => {
                                send_error.set(Some(e.to_string()));
                            }
                        }
                        match get_messages(&access_token, &team_id, &channel_id, &users_list()).await {
                            Ok(updated_messages) => {
                                messages_list.set(Some(updated_messages).clone());
                                info!("Messages update successful");
                            }
                            Err(e) => {
                                send_error.set(Some(e.to_string()));
                                info!("Messages update failed: {}", e);
                            }
                        }
                    });
                    break;
                }
            }
        }
        is_loading.set(false);
    };

    let mut handle_remove_reaction_reply = move |user_lock_api: Arc<Mutex<User>>| {
        let team_id = selected_team_id().as_ref().and_then(|team| team.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let channel_id = selected_channel_id().as_ref().and_then(|channel| channel.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let message_id = selected_message_id().as_ref().and_then(|message| message.get("replyFromId")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let reply_id = selected_message_id().as_ref().and_then(|reply| reply.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let user_id = selected_user_id().as_ref().and_then(|id| id.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();

        if let Some(reactions) = selected_message_id().as_ref().and_then(|reply| reply.get("reactions")).and_then(|r| r.as_array()) {
            for reaction in reactions {
                let reaction_user_id = reaction.get("user").and_then(|user| user.get("id")).and_then(|id| id.as_str()).unwrap_or("").to_string();
                if reaction_user_id == user_id {
                    let emoji = reaction.get("emoji").and_then(|v| v.as_str()).unwrap_or("");
                    block_on(async move {
                        let user_lock_api = user_lock_api.lock().await;
                        let access_token = user_lock_api.ms_teams.access_token.clone();
                
                        match remove_reaction_reply(&access_token, &team_id, &channel_id, &message_id, &reply_id, emoji).await {
                            Ok(()) => {
                            }
                            Err(e) => {
                                send_error.set(Some(e.to_string()));
                            }
                        }
                        match get_messages(&access_token, &team_id, &channel_id, &users_list()).await {
                            Ok(updated_messages) => {
                                messages_list.set(Some(updated_messages).clone());
                                info!("Messages update successful");
                            }
                            Err(e) => {
                                send_error.set(Some(e.to_string()));
                                info!("Messages update failed: {}", e);
                            }
                        }
                    });
                    break;
                }
            }
        }
        is_loading.set(false);
    };

    rsx! {
        div {
            class: "ms-teams-right-panel",

            // List of Messages/Posts
            div {
                class: "ms-teams-message-list",
                ul {
                    for message in messages_array.clone() {
                        if let message_copy = message.clone() {
                            li {
                                class: "ms-teams-post-container",
                                // The Container for Messages/Posts
                                div {
                                    class: "ms-teams-message",
                                    // Contains Profile Picture, User, and Time
                                    div {
                                        class: "ms-teams-user-header",
                                        div {
                                            class: "ms-teams-user-picture",
                                            if let Some(image_url) = message.clone().get("user").and_then(|u| u.get("profilePicture")).and_then(|v| v.as_str()) {
                                                img {
                                                    src: "{image_url}",
                                                    alt: "User Icon",
                                                    class: "ms-teams-user-icon-img"
                                                }
                                            }
                                        }
                                        span {
                                            class: "ms-teams-user-info",
                                            {message.get("user").and_then(|u| u.get("displayName")).and_then(|v| v.as_str()).unwrap_or("Unknown User")}
                                        }
                                        span {
                                            class: "ms-teams-message-time",
                                            {format_timestamp(message.get("time").and_then(|v| v.as_str()).unwrap_or(""))}
                                        }
                                    }
                                    // Contains the Subject of the Post
                                    div {
                                        class: "ms-teams-subject",
                                        {message.get("subject").and_then(|v| v.as_str()).unwrap_or("")}
                                    }
                                    // Contains the Content of the Post/Message
                                    div {
                                        class: "ms-teams-content",
                                        {message.get("content").and_then(|v| v.as_str()).unwrap_or("")}
                                    }
                                    // Contains the Reactions of the Message/Post
                                    div {
                                        class: "ms-teams-reactions",
                                        style: "position: relative",
                                        span {
                                            class: "ms-teams-reaction-list",
                                            for reaction in message.clone().get("reactions").and_then(|r| r.as_array()).unwrap_or(&vec![]).clone() {
                                                if let message_copy_clone = message_copy.clone() {
                                                    button {
                                                        class: "ms-teams-reaction-item",
                                                        disabled: is_loading(),
                                                        onclick: move |_| {
                                                            is_loading.set(true);
                                                            show_emoji_list.set(false);
                                                            show_reply_emoji_list.set(false);
                                                            show_reply_input.set(false);
                                                            reply_input.set("".to_string());
                                                            selected_message_id.set(Some(message_copy_clone.clone()));
                                                            handle_remove_reaction(Arc::clone(&user_lock()));
                                                        },
                                                        {reaction.get("emoji").and_then(|v| v.as_str()).unwrap_or("")}
                                                        " "
                                                        span {
                                                            class: "ms-teams-reaction-count",
                                                            {reaction.get("count").and_then(|v| v.as_i64()).unwrap_or(1).to_string()}
                                                        },
                                                    }
                                                }
                                            }
                                        }
                                        if let message_copy_clone = message_copy.clone() {
                                            button {
                                                class: "ms-teams-reaction-button",
                                                onclick: move |_| {
                                                    if show_reaction_list() {
                                                        if selected_message_id().as_ref().and_then(|message| message.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string() == message_copy_clone.clone().get("id").and_then(|v| v.as_str()).unwrap_or("").to_string() {
                                                            selected_message_id.set(None);
                                                            show_reaction_list.set(false);
                                                        }
                                                        else {
                                                            selected_message_id.set(Some(message_copy_clone.clone()));
                                                        }
                                                    }
                                                    else {
                                                        show_emoji_list.set(false);
                                                        show_reply_emoji_list.set(false);
                                                        show_reply_input.set(false);
                                                        reply_input.set("".to_string());
                                                        selected_message_id.set(Some(message_copy_clone.clone()));
                                                        show_reaction_list.set(true);
                                                    }
                                                },
                                            }
                                        }
                                        if show_reaction_list() {
                                            if selected_message_id().as_ref().and_then(|message| message.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string() == message.clone().get("id").and_then(|v| v.as_str()).unwrap_or("").to_string() {
                                                div {
                                                    class: "ms-teams-emoji-list",
                                                    for emoji in ["😁", "😂", "😍", "👍", "❤️", "😮", "😢", "😡"] {
                                                        button {
                                                            class: "ms-teams-emoji",
                                                            disabled: is_loading(),
                                                            onclick: move |_| {
                                                                is_loading.set(true);
                                                                show_reaction_list.set(false);
                                                                handle_send_reaction(emoji, Arc::clone(&user_lock()));
                                                            },
                                                            "{emoji}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div {
                                    class: "ms-teams-post-divider"
                                }
                                div {
                                    class: "ms-teams-replies-container",
                                    // Section for the Replies to each Message/Post
                                    for reply in message.clone().get("replies").and_then(|r| r.as_array()).unwrap_or(&vec![]).clone() {
                                        div {
                                            class: "ms-teams-reply",
                                            // Contains Profile Picture, User, and Time
                                            div {
                                                class: "ms-teams-user-header",
                                                div {
                                                    class: "ms-teams-user-picture",
                                                    if let Some(image_url) = reply.clone().get("user").and_then(|u| u.get("profilePicture")).and_then(|v| v.as_str()) {
                                                        img {
                                                            src: "{image_url}",
                                                            alt: "User Icon",
                                                            class: "ms-teams-user-icon-img"
                                                        }
                                                    }
                                                }
                                                span {
                                                    class: "ms-teams-user-info-reply",
                                                    {reply.get("user").and_then(|u| u.get("displayName")).and_then(|v| v.as_str()).unwrap_or("Unknown User")}
                                                }
                                                span {
                                                    class: "ms-teams-message-time-reply",
                                                    {format_timestamp(reply.get("time").and_then(|v| v.as_str()).unwrap_or(""))}
                                                }
                                            }
                                            // Contains Content of Reply
                                            div {
                                                class: "ms-teams-content-reply",
                                                {reply.get("content").and_then(|v| v.as_str()).unwrap_or("")}
                                            }
                                            // Contains Reactions of Reply
                                            div {
                                                class: "ms-teams-reactions",
                                                style: "position: relative",
                                                span {
                                                    class: "ms-teams-reaction-list",
                                                    for reaction in reply.clone().get("reactions").and_then(|r| r.as_array()).unwrap_or(&vec![]) {
                                                        if let reply_copy = reply.clone() {
                                                            button {
                                                                class: "ms-teams-reaction-item",
                                                                disabled: is_loading(),
                                                                onclick: move |_| {
                                                                    is_loading.set(true);
                                                                    show_emoji_list.set(false);
                                                                    show_reply_emoji_list.set(false);
                                                                    show_reply_input.set(false);
                                                                    reply_input.set("".to_string());
                                                                    selected_message_id.set(Some(reply_copy.clone()));
                                                                    handle_remove_reaction_reply(Arc::clone(&user_lock()));
                                                                },
                                                                {reaction.get("emoji").and_then(|v| v.as_str()).unwrap_or("")}
                                                                " "
                                                                span {
                                                                    class: "ms-teams-reaction-count",
                                                                    {reaction.get("count").and_then(|v| v.as_i64()).unwrap_or(1).to_string()}
                                                                },
                                                            }
                                                        }
                                                    }
                                                }
                                                button {
                                                    class: "ms-teams-reaction-button",
                                                    onclick: move |_| { 
                                                        if show_reaction_list() {
                                                            if selected_message_id().as_ref().and_then(|reply| reply.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string() == reply.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string() {
                                                                selected_message_id.set(None);
                                                                show_reaction_list.set(false);
                                                            }
                                                            else {
                                                                selected_message_id.set(Some(reply.clone()));
                                                            }
                                                        }
                                                        else {
                                                            show_emoji_list.set(false);
                                                            show_reply_emoji_list.set(false);
                                                            show_reply_input.set(false);
                                                            reply_input.set("".to_string());
                                                            selected_message_id.set(Some(reply.clone()));
                                                            show_reaction_list.set(true);
                                                        }
                                                    },
                                                }
                                                if show_reaction_list() {
                                                    if selected_message_id().as_ref().and_then(|reply| reply.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string() == reply.clone().get("id").and_then(|v| v.as_str()).unwrap_or("").to_string() {
                                                        div {
                                                            class: "ms-teams-emoji-list",
                                                            for emoji in ["😁", "😂", "😍", "👍", "❤️", "😮", "😢", "😡"] {
                                                                button {
                                                                    class: "ms-teams-emoji",
                                                                    disabled: is_loading(),
                                                                    onclick: move |_| {
                                                                        is_loading.set(true);
                                                                        show_reaction_list.set(false);
                                                                        handle_send_reaction_reply(emoji, Arc::clone(&user_lock()));
                                                                    },
                                                                    "{emoji}"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if let message_copy_clone = message_copy.clone() {
                                        button {
                                            class: "ms-teams-reply-button",
                                            onclick: move |_| {
                                                if show_reply_input() {
                                                    if selected_message_id().as_ref().and_then(|message| message.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string() == message_copy_clone.clone().get("id").and_then(|v| v.as_str()).unwrap_or("").to_string() {
                                                        show_reply_emoji_list.set(false);
                                                        reply_input.set("".to_string());
                                                        selected_message_id.set(None);
                                                        show_reply_input.set(false);
                                                    }
                                                    else {
                                                        show_reply_emoji_list.set(false);
                                                        reply_input.set("".to_string());
                                                        selected_message_id.set(Some(message_copy_clone.clone()));
                                                    }
                                                }
                                                else {
                                                    show_emoji_list.set(false);
                                                    show_reaction_list.set(false);
                                                    show_reply_emoji_list.set(false);
                                                    reply_input.set("".to_string());
                                                    selected_message_id.set(Some(message_copy_clone.clone()));
                                                    show_reply_input.set(true);
                                                }
                                            },
                                            "Reply"
                                        }
                                    }
                                    //need extra check here
                                    if show_reply_input() && selected_message_id().as_ref().and_then(|message| message.get("id")).and_then(|v| v.as_str()) == message_copy.clone().get("id").and_then(|v| v.as_str()) {
                                        div {
                                            class: "ms-teams-reply-input-container",
                                            position: "relative",
                                            input {
                                                class: "ms-teams-reply-input",
                                                value: "{reply_input()}",
                                                placeholder: "Type your reply here...",
                                                onclick: move |_| {
                                                    show_reaction_list.set(false);
                                                    show_reply_emoji_list.set(false);
                                                },
                                                oninput: move |event| {
                                                    reply_input.set(event.value());
                                                },
                                            }
                                            button {
                                                class: "ms-teams-reply-emoji-button",
                                                onclick: move |_| {
                                                    show_reaction_list.set(false);
                                                    show_reply_emoji_list.set(!show_reply_emoji_list());
                                                },
                                                "😀"
                                            }
                                            if show_reply_emoji_list() {
                                                div {
                                                    class: "ms-teams-emoji-list",
                                                    for emoji in ["😁", "😂", "😍", "👍", "❤️", "😮", "😢", "😡"] {
                                                        span {
                                                            class: "ms-teams-emoji",
                                                            onclick: move |_| {
                                                                reply_input.set(format!("{}{}", reply_input(), emoji));
                                                                show_reply_emoji_list.set(false);
                                                            },
                                                            "{emoji}"
                                                        }
                                                    }
                                                }
                                            }
                                            button {
                                                class: "ms-teams-reply-send-button",
                                                disabled: is_loading(),
                                                onclick: move |_| {
                                                    is_loading.set(true);
                                                    show_reply_input.set(false);
                                                    handle_send_reply(Arc::clone(&user_lock()));
                                                },
                                                "Send"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Send Messages Text Box
            div {
                class: "ms-teams-message-system-container",
                div {
                    class: "ms-teams-subject-toggle-container",
                    label {
                        class: "ms-teams-subject-toggle-label",
                        input {
                            r#type: "checkbox",
                            checked: "{show_subject_input}",
                            onchange: move |_| {
                                message_subject_input.set("".to_string());
                                show_subject_input.set(!show_subject_input());
                            },
                        }
                        span {
                            style: "margin-top: 0.01rem;",
                            "Add Subject"
                        }
                    }
                    if show_subject_input() {
                        div {
                            class: "ms-teams-subject-input-container",
                            input {
                                class: "ms-teams-subject-input",
                                value: "{message_subject_input}",
                                placeholder: "Enter Subject... (Optional)",
                                onclick: move |_| {
                                    selected_message_id.set(None);
                                    show_reply_input.set(false);
                                    reply_input.set("".to_string());
                                    show_reaction_list.set(false);
                                    show_reply_emoji_list.set(false);
                                    show_emoji_list.set(false);
                                },
                                oninput: move |event| {
                                    message_subject_input.set(event.value());
                                },
                            }
                        }
                    }
                }
                div {
                    class: "ms-teams-message-container",
                    div {
                        class: "ms-teams-message-action-container",
                        input {
                            class: "ms-teams-message-input",
                            value: "{message_input}",
                            placeholder: "Type here...",
                            onclick: move |_| {
                                selected_message_id.set(None);
                                show_reply_input.set(false);
                                reply_input.set("".to_string());
                                show_reaction_list.set(false);
                                show_reply_emoji_list.set(false);
                                show_emoji_list.set(false);
                            },
                            oninput: move |event| {
                                message_input.set(event.value());
                            },
                        }
                        button {
                            class: "ms-teams-emoji-button",
                            onclick: move |_| {
                                selected_message_id.set(None);
                                show_reply_input.set(false);
                                reply_input.set("".to_string());
                                show_reaction_list.set(false);
                                show_reply_emoji_list.set(false);
                                show_emoji_list.set(!show_emoji_list());
                            },
                            "😀"
                        }
                        if show_emoji_list() {
                            div {
                                class: "ms-teams-emoji-list",
                                for emoji in ["😁", "😂", "😍", "👍", "❤️", "😮", "😢", "😡"] {
                                    span {
                                        class: "ms-teams-emoji",
                                        onclick: move |_| {
                                            message_input.set(format!("{}{}", message_input(), emoji));
                                            show_emoji_list.set(false);
                                        },
                                        "{emoji}"
                                    }
                                }
                            }
                        }
                        button {
                            class: "ms-teams-send-button",
                            disabled: is_loading(),
                            onclick: move |_| {
                                is_loading.set(true);
                                handle_send_message(Arc::clone(&user_lock()));
                            },
                            "Send"
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct EmptyStruct {} // Empty struct to use for coroutines (when you don't need to send anything into the coroutine)

fn format_timestamp(timestamp: &str) -> String {
    // Parse the timestamp string into a DateTime object
    let parsed_timestamp = DateTime::parse_from_rfc3339(timestamp).unwrap_or_else(|_| Utc::now().into());
    
    // Convert to local time
    let local_timestamp = parsed_timestamp.with_timezone(&Local);

    // Format the date into a readable format, e.g., "Sep 26, 2024 12:45 PM"
    local_timestamp.format("%b %d, %Y %I:%M %p").to_string()
}