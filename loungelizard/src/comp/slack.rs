use dioxus:: prelude::*;
use chrono::{Local};
use mongodb::action::Shutdown;
use tracing::info;
//use dioxus_logger::tracing::{info, error, warn};
use std::{process::id, sync::Arc};
use tokio::sync::Mutex;
use crate::api::{mongo_format::mongo_structs::*, slack::{self, emoji::*}};
use slack_morphism::prelude::*;
use futures::{executor::block_on, StreamExt};
// ! Message Component 

#[component]
pub fn CH_DM_Component(
    channel_info: SlackChannelInfo,
    selected_channel: Signal<Option<SlackChannelInfo>>,
) -> Element {
    // Determine the background color and alignment based on the "me" prop
    let _user_lock = use_context::<Signal<Arc<Mutex<User>>>>();

    let background = 
    match selected_channel().clone()
    {
        Some(channel) => {
            if channel.id == channel_info.id {
                "#EAD01C" // Light blue
            } else {
                "white" // Purple
            }
        },
        None => "white"
    };

    let channel_name = 
        use_signal(||
            match channel_info.clone().name
            {
                Some(name) => name.clone(),
                None => "".to_string()
            }
        );

    let handle_click = move |_|{
        selected_channel.set(Some(channel_info.clone())); // Clone the channel_info variable
        
    };

    rsx! {
        div {
            li {
                style: format!("margin-bottom: 10px; cursor: pointer; color:{};", background),
                onclick: handle_click,
                "{channel_name}",
                
            }
        }
    }
}

// ! Custom Struct to hold both message type and ensure we can use them both 
// ! in one parameter 
#[derive(Debug, PartialEq)]
pub struct MessageComp {
    pub mess_h: Option<SlackHistoryMessage>,
    pub mess_e: Option<SlackMessageEvent>,
}
impl Clone for MessageComp {
    fn clone(&self) -> Self {
        MessageComp {
            mess_h: self.mess_h.clone(),
            mess_e: self.mess_e.clone(),
        }
    }
}


pub async fn remove_reaction(
    reaction    :String,
    origin      :SlackMessageOrigin,
    token_s     :String,
     
)
{
    // Create a new Slack client 
    let client  = 
    SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));

    // Create a new token from the environment variable `SLACK_CONFIG_TOKEN`
    let token: SlackApiToken = SlackApiToken::new(token_s.into());

    // Create a new session with the client and the token
    let session = client.open_session(&token);

    let ts = origin.ts;
    let channel_id = origin.channel.unwrap();

    let remove_request = 
        SlackApiReactionsRemoveRequest::new(reaction.clone().into())
            .with_channel(channel_id.into())
            .with_timestamp(ts);
    
   
    let r= session.reactions_remove(&remove_request).await;
    

}

pub async fn add_reaction(
    reaction    :String,
    origin      :SlackMessageOrigin,
    token_s     :String,
     
)
{
    // Create a new Slack client 
    let client  = 
    SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));

    // Create a new token from the environment variable `SLACK_CONFIG_TOKEN`
    let token: SlackApiToken = SlackApiToken::new(token_s.into());

    // Create a new session with the client and the token
    let session = client.open_session(&token);

    let ts = origin.ts;
    let channel_id = origin.channel.unwrap();

    let add_request = 
        SlackApiReactionsAddRequest::new(
            channel_id.into(),
            SlackReactionName::new(reaction.clone()),
            ts.clone());

    let _response = session.reactions_add(&add_request).await;
}

#[component]
fn EmojiPickerComponent(on: Signal<bool>, origin: SlackMessageOrigin) -> Element {
    let origin_clone = origin.clone(); // Clone origin here

    let user_lock: Signal<Arc<Mutex<User>>> = use_context::<Signal<Arc<Mutex<User>>>>();
    let user_lockToken = Arc::clone(&user_lock());

    let slack_token = use_signal(||
        block_on(
            async{
                let user = user_lockToken.lock().await;
                user.slack.user.token.clone()
            }
        )
    );

    let send_task = use_coroutine(|mut rx|{
        let slack_token = slack_token.to_owned(); 
            async move {
                while let Some(emoji) = rx.next().await {
                    let _ = add_reaction(
                                emoji, 
                                origin_clone.clone(),
                                slack_token().clone()
                            ).await;
                }

            }
        }
    );

    rsx! {
        
        div {
            style: "position: relative; display: inline-block;",
            // Emoji picker pane (modal-like)
            div {
                style: "
                    position: absolute;
                    top: -10px;
                    left: -60px;
                    display: grid;
                    grid-template-columns: repeat(5, 1fr);
                    gap: 8px;
                    padding: 12px;
                    border: 1px solid #ddd;
                    background-color: #f5f5f5;
                    border-radius: 8px;
                    box-shadow: 0px 4px 12px rgba(0, 0, 0, 0.1);
                    max-width: calc(100vw - 20px); 
                    max-height: 150px;   
                    overflow-y: auto;   
                    z-index: 100;
                ",
                for (label, emoji) in EMOJIS.entries(){
                    div {
                        style: "
                            font-size: 1.5em;
                            padding: 6px;
                            cursor: pointer;
                            transition: background-color 0.2s;
                        ",
                        onclick: move |_| send_task.send(label.to_string())
                        , // Select emoji on click
                        "{emoji}"
                    }
                }
            }
        }
    }
}

// ! Custom Message Component 
// ! This component is used to display custom messages
#[component]
pub fn CustomMessageComponent(
    message: MessageComp,
    user_id: String,
    current_selected_id: Signal<Option<String>>,
) -> Element {

    let user_lock: Signal<Arc<Mutex<User>>> = use_context::<Signal<Arc<Mutex<User>>>>();
    let user_lockToken = Arc::clone(&user_lock());

    let mut show_pane = use_signal(|| false); // Default edit mode is false; 
    let mut show_reactions = use_signal(|| false);
    let mut show_edit = use_signal(|| false);
    let mut current_reaction_name = use_signal(|| "".to_string());

    // ! Variables that will be needed for styling and displaying the message
    let mut origin  = Option::None;
    let mut sender  = Option::None;
    let mut content = Option::None;
    let mut edited  = Option::None;
    let mut subtype = Option::None;

    let show_pane_fn = ||{
        show_pane() || show_reactions() || show_edit()
        // TODO Implement a uinque id for each message and conpare to current 1.  && !occupied()
    };

    // ! Unpack the message  
    match message{
        MessageComp { mess_h: Some(message_h), .. } => {
            origin = Some(message_h.origin.clone());
            sender = Some(message_h.sender.clone());
            content = Some(message_h.content.clone());
            edited = message_h.edited.clone();
            subtype = message_h.subtype.clone();
        },
        MessageComp { mess_e: Some(message_e), .. } => {
            origin = Some(message_e.origin.clone());
            sender = Some(message_e.sender.clone());
            content = Some(message_e.content.clone().unwrap());
            subtype = message_e.subtype.clone();
            match message_e.message{
                Some(message) => {
                    edited = message.edited.clone();
                },
                None => {
                    edited = None;
                }
            }
        },
        _ => {
            return rsx!();
        }
    }


    let origin_clone_mouse_enter = origin.clone();
    let origin_clone_mouse_leave = origin.clone();

    let origin_clone = origin.clone();

    use_effect( move || {
        let reaction_name = current_reaction_name();

        if !reaction_name.is_empty() {
            block_on(
                async{
                    let user = user_lockToken.lock().await;
                    let slack_token = user.slack.user.token.clone();

                    match origin_clone.clone(){

                        Some(origin) => {
                            let _ = remove_reaction(
                                reaction_name, 
                                origin.clone(),
                                slack_token.clone()
                            ).await;
                        },
                        None => {}
                    }

                
            })
        }
        
    });

    rsx! {
        li {
            
            style:format!(
                        "display: flex; flex-direction: column; align-items: flex-start; margin: 10px;{}",
                        if show_pane_fn() { "background-color: #3a3a3a;" } else { "" }
                    ),
            onmouseenter: move |_| {
                if current_selected_id().is_none() {
                    show_pane.set(true);
                    current_selected_id.set(Some(origin_clone_mouse_enter.clone().unwrap().ts.to_string()));
                }
            },
            onmouseleave: move |_| {
                
                if !show_reactions() && !show_edit() {
                    if let Some(id) = current_selected_id() {
                        let ts_id = origin_clone_mouse_leave.clone().unwrap().ts.to_string();
                        if  id == ts_id {
                            show_pane.set(false);
                            current_selected_id.set(None);
                        }
                          
                    }
                }
                    
            },
            div {
                style: "font-size: 0.8em; color: gray; margin-top: 4px; display: flex; align-items: center;", // Styling for user and timestamp
                match &sender.as_ref().unwrap().user {
                    // TODO Change to usernames 
                    Some(username) => rsx!(
                        span {
                            style: "margin-right: 8px; font-size: 1.2em; font-weight: bold; color: white;",
                            if username.to_string() == user_id {
                                {format!("You")}
                            } else {
                                {format!("{}", username)}
                            },
                        },
                        span {
                            style: "margin-right: 4px;",
                            {
                                match &origin.as_ref().unwrap().ts.to_date_time_opt(){
                                    Some(timestamp) => {
                                        format!("{}", timestamp.with_timezone(&Local).format("%I:%M %p"))
                                    },
                                    None => {
                                        format!("{}", "Unknown")
                                    }
                                }
                            }
                        },
                    ),
                    None => rsx!(
                        span {
                            style: "margin-right: 4px;",
                            "Unknown",
                        },
                        span {
                            style: "margin-right: 4px;",
                            {
                                match &origin.as_ref().unwrap().ts.to_date_time_opt(){
                                    Some(timestamp) => {
                                        format!("{}", timestamp.with_timezone(&Local).format("%I:%M %p"))
                                    },
                                    None => {
                                        format!("{}", "Unknown")
                                    }
                                }
                            }
                        },
                    ),
                },
                if let Some(_) = edited {
                    div {
                        style: "display: flex; flex-direction: column; align-items: flex-end; font-size: 0.8em; color: red;",
                        "(edited)"
                    }
                }
            },
            div{
                style: "display: flex; cursor: pointer; justify-content: space-between; align-items: center; width: 100%;",
                div {
                    style: match &sender.as_ref().unwrap().user {
                        Some(user) => format!(
                            "padding: 8px; border-radius: 8px; background-color: {}; color: white; max-width: 100%;",
                            if user.to_string() == user_id {"#6CA6E1"} else {"#8A2BE2"}
                        ),
                        None => String::new(), // Return an empty string if there is no user
                    },
                    match &content.as_ref().unwrap().text {
                        Some(text) => format!("{}", text), // Display the message content if available
                        None => "No content available".to_string(), // Fallback text if there is no content
                    }
                    match &content.as_ref().unwrap().reactions {
                        
                        Some(reactions) => 
                            rsx!(
                                div{
                                    style: "display: flex; align-items: center;",
                                    for reaction in reactions {
                                        
                                        li{
                                                style: "border: 1px solid #ddd; display: flex; align-items: center; 
                                                    padding: 4px 6px; border-radius: 12px; background-color: #3a3a3a; 
                                                    font-size: 0.9em; cursor: pointer; transition: background-color 0.2s;",
                                                onclick: {
                                                    let reaction_name = reaction.name.to_string().clone();
                                                    move |_| current_reaction_name.set(reaction_name.clone())
                                                },
                                                span{
                                                    
                                                    style: "max-width: 18px; height: 18px; margin-right: 4px;",
                                                    {
                                                        format!("{}", get_emoji(reaction.name.to_string().as_str()))
                                                    }
                                                },
                                                span{
                                                    style: "margin-right: 4px; font-weight: bold; color: white;",
                                                    {
                                                        format!(":{}", reaction.count.to_string())
                                                    }
                                                },
                                                
                                        }
                                    
                                    }
                                }
                            ) 
                        
                        ,
                        None => rsx!()
                    }
                },
                if show_pane_fn() {
                    div{
                        
                        style: "display: flex; flex-direction: column; align-items: flex-end; margin-left: auto; cursor: pointer;",
                       // Reaction Button

                       div{
                            style: "display: flex; gap: 8px; margin-bottom: 8px;",
                            button {
                                style: "
                                background-color: #003366; 
                                color: white; 
                                border: none; 
                                border-radius: 12px; 
                                padding: 6px 12px; 
                                font-size: 14px; 
                                cursor: pointer;
                                margin-left: auto; 
                                ",  
                                onclick: move |_| show_reactions.set(!show_reactions()),
                                "➕ Add Reaction"
                            },
                            if subtype.as_ref().is_none() {
                                button{
                                    style: "
                                    background-color: #003366; 
                                    color: white; 
                                    border: none; 
                                    border-radius: 12px; 
                                    padding: 6px 12px; 
                                    font-size: 14px; 
                                    cursor: pointer;
                                    margin-left: auto;
                                    ", 
                                    "Edit Message"
                                }
                            }
                            
                        },
                        
                        if show_reactions() {
                            div{
                                style: "margin-right: auto;",
                                EmojiPickerComponent{
                                    on: show_reactions.clone(),
                                    origin: origin.clone().unwrap()
                                }
                            }
                            
                        }
                        
                    }
                    
                    
                
                    
                },
                
            },
        }
    }
}

