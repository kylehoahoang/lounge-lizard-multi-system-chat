use dioxus:: prelude::*;
use chrono::Local;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::api::{mongo_format::mongo_structs::*, slack::{self, emoji::*}};
use slack_morphism::prelude::*;
use futures::{executor::block_on, StreamExt};
use std::collections::HashMap;
use serde_json::Value;
use base64::encode;
use reqwest::Client as ReqwestClient;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::Error;
// ! Message Component 

#[component]
/// Creates a channel component which is rendered as a list item (li)
/// It takes two parameters, the channel_info and the selected_channel
/// The selected_channel is a signal which is used to determine the background color of the component
/// If the channel_info matches the selected_channel, the background is yellow, otherwise it is white
/// The component also has a click event handler, which sets the selected_channel to the channel_info when clicked
pub fn CH_DM_Component(
    channel_info: SlackChannelInfo,
    selected_channel: Signal<Option<SlackChannelInfo>>,
) -> Element {
    // Get the user context from the context
    let _user_lock = use_context::<Signal<Arc<Mutex<User>>>>();

    // Determine the background color based on the selected channel
    let background = 
    match selected_channel().clone()
    {
        // If the channel_info matches the selected_channel, set the background color to yellow
        Some(channel) => {
            if channel.id == channel_info.id {
                "#EAD01C" // Yellow
            } else {
                "white" // White
            }
        },
        // If there is no selected channel, set the background color to white
        None => "white"
    };

    // Get the channel name from the channel_info
    let channel_name = 
        use_signal(||
            match channel_info.clone().name
            {
                // If the channel has a name, clone the name
                Some(name) => name.clone(),
                // If the channel does not have a name, return an empty string
                None => "".to_string()
            }
        );

    // Define a click event handler
    let handle_click = move |_|{
        // Set the selected_channel to the channel_info when clicked
        selected_channel.set(Some(channel_info.clone())); // Clone the channel_info variable
        
    };

    // Render the component
    rsx! {
        div {
            // Render a list item (li)
            li {
                // Set the style based on the background color
                style: format!("margin-bottom: 10px; cursor: pointer; color:{};", background),
                // Attach the click event handler to the list item
                onclick: handle_click,
                // Render the channel name
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
    reaction: String,
    origin: SlackMessageOrigin,
    token_s: String,
) {
    // Create a new Slack client using the SlackClientHyperConnector.
    // This will establish the connection with Slack's API.
    let client = SlackClient::new(
        SlackClientHyperConnector::new().expect("failed to create hyper connector")
    );

    // Convert the provided token string into a SlackApiToken object.
    // This token is required for authentication with Slack's API.
    let token: SlackApiToken = SlackApiToken::new(token_s.into());

    // Open a new session with the Slack client using the provided token.
    // This session will be used to send requests to the Slack API.
    let session = client.open_session(&token);

    // Extract the timestamp from the origin of the message.
    // The timestamp identifies the specific message to which the reaction belongs.
    let ts = origin.ts;

    // Extract the channel ID from the origin of the message.
    // The channel ID tells us where the message is located.
    let channel_id = origin.channel.unwrap();

    // Create a request to remove a reaction from a message.
    // The request requires the reaction name, channel ID, and timestamp.
    let remove_request = SlackApiReactionsRemoveRequest::new(reaction.clone().into())
        .with_channel(channel_id.into())
        .with_timestamp(ts);

    // Send the remove reaction request via the session.
    // This will asynchronously communicate with Slack's API to remove the reaction.
    let _response = session.reactions_remove(&remove_request).await;
}


/// Asynchronously adds a reaction to a message on Slack.
///
/// # Parameters
///
/// * `reaction`: The name of the reaction to add.
/// * `origin`: The origin of the message to which the reaction should be added.
/// * `token_s`: The Slack bot token as a string.
///
/// # Process
///
/// 1. Creates a new Slack client.
/// 2. Converts the provided token string into a `SlackApiToken` object.
/// 3. Opens a new session with the client and token.
/// 4. Extracts the timestamp and channel ID from the origin of the message.
/// 5. Creates a request to add a reaction to a message with the given timestamp and channel ID.
/// 6. Sends the request to the Slack API to add the reaction.
pub async fn add_reaction(
    reaction    :String,
    origin      :SlackMessageOrigin,
    token_s     :String,
     
)
{
    // Step 1: Create a new Slack client
    let client  = 
    SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));

    // Step 2: Convert the provided token string into a SlackApiToken object
    let token: SlackApiToken = SlackApiToken::new(token_s.into());

    // Step 3: Open a new session with the client and token
    let session = client.open_session(&token);

    // Step 4: Extract the timestamp and channel ID from the origin of the message
    let ts = origin.ts;
    let channel_id = origin.channel.unwrap();

    // Step 5: Create a request to add a reaction to a message with the given timestamp and channel ID
    let add_request = 
        SlackApiReactionsAddRequest::new(
            channel_id.into(),
            SlackReactionName::new(reaction.clone()),
            ts.clone());

    // Step 6: Send the request to the Slack API to add the reaction
    let _response = session.reactions_add(&add_request).await;
}

/// Edit a message in a Slack channel.
///
/// # Parameters
///
/// * `content`: The new content of the message
/// * `origin`: The origin of the message to be edited
/// * `token_s`: The Slack bot token
///
/// # Steps
///
/// 1. Create a new Slack client
/// 2. Convert the provided token string into a `SlackApiToken` object
/// 3. Open a new session with the client and token
/// 4. Extract the timestamp and channel ID from the origin of the message
/// 5. Create a request to edit a message with the given timestamp and channel ID
/// 6. Send the request to the Slack API to edit the message
pub async fn edit_message_fn(
    content     :SlackMessageContent,
    origin      :SlackMessageOrigin,
    token_s     :String,
)
{
    // Step 1: Create a new Slack client
    let client  = 
    SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));

    // Step 2: Convert the provided token string into a SlackApiToken object
    let token: SlackApiToken = SlackApiToken::new(token_s.into());

    // Step 3: Open a new session with the client and token
    let session = client.open_session(&token);

    // Step 4: Extract the timestamp and channel ID from the origin of the message
    let ts = origin.ts;
    let channel_id = origin.channel.unwrap();

    // Step 5: Create a request to edit a message with the given timestamp and channel ID
    let edit_request = 
        SlackApiChatUpdateRequest::new(
            channel_id.into(),
            content.clone(),
            ts.clone() // Origin.content
        )
        .with_as_user(true)
        .with_parse("full".to_string());

    // Step 6: Send the request to the Slack API to edit the message
    let _response = session.chat_update(&edit_request).await;
    //println!("Edit response: {:#?}", content);

    
}

#[component]
fn EmojiPickerComponent(on: Signal<bool>, origin: SlackMessageOrigin) -> Element {
    // Clone the origin to use in asynchronous operations
    let origin_clone = origin.clone();

    // Retrieve the user lock signal from the context
    let user_lock: Signal<Arc<Mutex<User>>> = use_context::<Signal<Arc<Mutex<User>>>>();
    // Clone the user lock to access the user data
    let user_lockToken = Arc::clone(&user_lock());

    // Create a signal to store the Slack token
    let slack_token = use_signal(|| {
        // Execute an asynchronous block to retrieve the token data
        block_on(async {
            // Acquire a lock on the user data
            let user = user_lockToken.lock().await;
            // Clone the user's Slack token
            user.slack.user.token.clone()
        })
    });

    // Set up a coroutine to handle sending emojis as reactions
    let send_task = use_coroutine(|mut rx| {
        // Clone the Slack token to use in the async block
        let slack_token = slack_token.to_owned();
        async move {
            // Continuously receive emoji selections
            while let Some(emoji) = rx.next().await {
                // Add a reaction to the Slack message with the selected emoji
                let _ = add_reaction(
                    emoji,
                    origin_clone.clone(),
                    slack_token().clone()
                ).await;
            }
        }
    });

    // Render the emoji picker component
    rsx! {
        // Outer container with relative positioning and inline-block display
        div {
            style: "position: relative; display: inline-block;",
            // Inner container for the emoji picker pane
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
                // Iterate over the emoji entries to display each emoji
                for (label, emoji) in EMOJIS.entries() {
                    // Individual emoji display with styling and click handler
                    div {
                        style: "
                            font-size: 1.5em;
                            padding: 6px;
                            cursor: pointer;
                            transition: background-color 0.2s;
                        ",
                        // When clicked, send the emoji label to the coroutine task
                        onclick: move |_| send_task.send(label.to_string()),
                        "{emoji}" // Display the emoji character
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
    incoming_user: Signal<HashMap<String, SlackUser>>
) -> Element {

    // Retrieve the user context from the application's context and clone it for different use cases
    let user_lock: Signal<Arc<Mutex<User>>> = use_context::<Signal<Arc<Mutex<User>>>>();
    let user_lockToken_reaction = Arc::clone(&user_lock()); // Clone for reaction-related operations
    let user_lockToken_edit = Arc::clone(&user_lock()); // Clone for message editing operations
    let user_lockToken_file = Arc::clone(&user_lock()); // Clone for file handling operations

    // Signals to control the display of different UI panes or modes
    let mut show_pane = use_signal(|| false); // Determines if a general pane is shown
    let mut show_reactions = use_signal(|| false); // Determines if the reaction pane is shown
    let mut show_edit = use_signal(|| false); // Determines if the edit message pane is shown

    // Signals for managing current state and user inputs
    let mut current_reaction_name = use_signal(|| "".to_string()); // Holds the name of the current reaction
    let mut edited_message = use_signal(|| "".to_string()); // Stores the edited message text
    let mut edit_message_send = use_signal(|| "".to_string()); // Stores the message text to be sent after editing

    // Signals for storing collections of media tags mapped to identifiers
    let mut img_tag_s: Signal<HashMap<String, (String, String)>> = use_signal(|| HashMap::new()); // Stores image tags and their attributes
    let mut audio_tag_s: Signal<HashMap<String, (String, String)>> = use_signal(|| HashMap::new()); // Stores audio tags and their attributes
    let mut video_tag_s: Signal<HashMap<String, (String, String)>> = use_signal(|| HashMap::new()); // Stores video tags and their attributes
    let mut code_tag_s: Signal<HashMap<String, (String, String)>> = use_signal(|| HashMap::new()); // Stores code block tags and their attributes
    let mut other_tag_s: Signal<HashMap<String,  String>> = use_signal(|| HashMap::new()); // Stores other types of tags and their attributes

    // Variables to hold different components of a message for styling and displaying purposes
    let mut origin  = Option::None; // The origin of the message (e.g., timestamp or ID)
    let mut sender  = Option::None; // The sender of the message
    let mut content = Option::None; // The content of the message
    let mut edited  = Option::None; // The edited state of the message
    let mut subtype = Option::None; // The subtype of the message (e.g., text, file)

    // Determine if any of the following panes are shown: general pane, reaction pane, or edit message pane
    let show_pane_fn = || {
        // If any of the following panes are shown, return true
        // This is used to conditionally render the message component
        show_pane() || show_reactions() || show_edit()
    };

    // Unpack the message content from the MessageComp struct
    match message {
        // If the message is a MessageHeader, unpack the origin, sender, content, edited status, and subtype
        MessageComp { mess_h: Some(message_h), .. } => {
            origin = Some(message_h.origin.clone());
            sender = Some(message_h.sender.clone());
            content = Some(message_h.content.clone());
            edited = message_h.edited.clone();
            subtype = message_h.subtype.clone();
            // ! Setting edit message 
        },
        // If the message is a MessageEvent, unpack the origin, sender, content, and subtype
        MessageComp { mess_e: Some(message_e), .. } => {
            origin = Some(message_e.origin.clone());
            sender = Some(message_e.sender.clone());
            content = Some(message_e.content.clone().unwrap());
            subtype = message_e.subtype.clone();
            match message_e.message {
                // If the message has a message property, unpack the edited status
                Some(message) => {
                    edited = message.edited.clone();
                },
                // If the message does not have a message property, set edited to None
                None => {
                    edited = None;
                }
            }
        },
        _ => {
            // If the message is neither a MessageHeader nor a MessageEvent, return an empty element
            return rsx!();
        }
    }

    // Determine if the user is allowed to edit the message
    // This is based on whether the sender of the message is the same as the current user
    let allow_edit = use_signal (|| {
        if sender.clone().unwrap().user.unwrap().to_string() == user_id {
            true
        }
        else {
            false
        }
    });

    // Clone the origin signal for use in the mouse enter and leave handlers
    let origin_clone_mouse_enter = origin.clone();
    let origin_clone_mouse_leave = origin.clone();

    // Clone the origin signal for use in the edit message and reaction handlers
    let origin_clone_edit = origin.clone();
    let origin_clone_reaction = origin.clone();

    // Clone the content signal for use in the message display
    let content_clone = content.clone();
    // Clone the content signal for use in the media pre-rendering
    let content_clone_files = content.clone();
    // Clone the content signal for use in the edit message feature
    let mut content_clone_edit = content.clone();
    // Pre-Render Media 
    use_effect(move ||{
        // This effect is used to pre-render media content within messages
        // It is used to fetch and store media content in signals that can be used to display the media
        // The files content is fetched from the content_clone_files signal
        // The user token is fetched from the user_lockToken_file signal
        let files_content = content_clone_files.clone();
        let token_clone = user_lockToken_file.clone();

        block_on(
            async {

                // If the files content is not empty, iterate over the files
                match files_content.unwrap().files{
                    Some(files) => {

                        // Fetch the user from the user_lockToken_file signal
                        let user = token_clone.lock().await.clone();

                        // Iterate over each file in the files array
                        for file in files{

                            // If the file has a URL private, fetch the image data
                            match file.url_private {
                                Some(item_url) => {
                                    // Fetch the image data from the private URL
                                    match file.filetype{
                                        // If the file type is an image type (png, jpg, jpeg, gif)
                                        Some(filetype) => {
                                            match filetype.to_string().as_str(){
                                                "png" | "jpg" | "jpeg" | "gif"  => {
                                                    // Fetch the image data from the private URL
                                                    let base64_image = fetch_image_with_bearer(item_url.as_str(), &user.slack.user.token).await.unwrap();
                                                    
                                                    // Store the image data in the img_tag_s signal
                                                    let img_tag = format!(
                                                        "data:image/{};base64,{}",
                                                        filetype.to_string(),
                                                        base64_image
                                                    );

                                                    img_tag_s.write().insert(file.name.unwrap(), (img_tag, file.url_private_download.unwrap().to_string()));
                                                },
                                                // If the file type is a video type (mp4, mov)
                                                "mp4" | "mov" => {
                                                    // Fetch the image data from the private URL
                                                    let base64_image = fetch_image_with_bearer(item_url.as_str(), &user.slack.user.token).await.unwrap();

                                                    // Store the video data in the video_tag_s signal
                                                    let file_in = 
                                                    match filetype.to_string().as_str(){
                                                        "mp4" => "mp4",
                                                        "mov" => "mov",
                                                        _ => ""
                                                    };

                                                    let video_tag = format!(
                                                        "data:video/{};base64,{}",
                                                        file_in.to_string(),
                                                        base64_image
                                                    );

                                                    video_tag_s.write().insert(file.name.unwrap(), (video_tag, file.url_private_download.unwrap().to_string()));

                                                },
                                                // If the file type is an audio type (mp3, wav, m4a)
                                                "mp3" | "wav" | "m4a" => {
                                                    // Fetch the image data from the private URL
                                                    let base64_image = fetch_image_with_bearer(item_url.as_str(), &user.slack.user.token).await.unwrap();
                                                    let file_in = 
                                                    match filetype.to_string().as_str(){
                                                        "mp3" => "mp3",
                                                        "wav" => "wav",
                                                        "m4a" => "mp4",
                                                        _ => ""
                                                    };

                                                    let audio_tag = format!(
                                                        "data:audio/{};base64,{}",
                                                        file_in.to_string(),
                                                        base64_image
                                                    );

                                                    audio_tag_s.write().insert(file.name.unwrap(), (audio_tag, file.url_private_download.unwrap().to_string()));
                                                },
                                                // If the file type is a code type (c, cmake, python)
                                                "c" | "cmake" | "python" => {
                                                    // Fetch the code data from the private URL
                                                    let file_lines  = fetch_code_from_url(item_url.as_str(), &user.slack.user.token).await.unwrap();
                                                    
                                                    // Store the code data in the code_tag_s signal
                                                    code_tag_s.write().insert(file.name.unwrap(), (file_lines, file.url_private_download.unwrap().to_string()));

                                                },
                                                // If the file type is unknown
                                                _ => {
                                                    
                                                    other_tag_s.write().insert(file.name.unwrap(), file.url_private_download.unwrap().to_string());
                                                }

                                            }
                                        },
                                        None => {} // No FileTypes 
                                    }
                                },
                                None => {}
                            }
                            
                        }
    
                    },
                    None => {} // No Files
                }
            }
        )
    });


    use_effect(move || {
        // Get the message from the edit_message_send signal
        let message = edit_message_send();

        // Clone the content_clone_edit signal and get a mutable reference to it
        let content_clone_edit_mut = content_clone_edit.as_mut().expect("Text Not found");
        
        // Set the text of the content_clone_edit signal to the message
        content_clone_edit_mut.text = Some(message.clone());
        
        // Get a mutable reference to the blocks of the content_clone_edit signal
        let blocks_mut = content_clone_edit_mut.blocks.as_mut();
        
        // Check if blocks_mut is Some
        if let Some(blocks) = blocks_mut {
            // Get a mutable reference to the first element of the blocks
            let first_block_mut = blocks.first_mut();
            
            // Check if the first block is a RichText block
            if let Some(SlackBlock::RichText(data)) = first_block_mut {
                // Get a mutable reference to the text of the RichText block
                let text = &mut data["elements"][0]["elements"][0]["text"];
                
                // Set the text of the RichText block to the message
                *text = Value::String(message.clone()); 
            }
        }
        // Clone the content_clone_edit signal to use in the async block
        let content_clone_edit_clone = content_clone_edit.clone();

        // Check if the message is not empty before proceeding
        if !message.is_empty() {
            // Block on the async block to edit the message
            block_on(async {
                // Acquire a lock on the user data to retrieve the Slack token
                let user = user_lockToken_edit.lock().await;
                let slack_token = user.slack.user.token.clone();

                // Clone the origin and match on its value
                match origin_clone_edit.clone() {
                    // If the origin is Some, proceed to edit the message
                    Some(origin) => {
                        // Call the edit_message_fn to update the message content on Slack
                        let _ = edit_message_fn(
                            content_clone_edit_clone.unwrap(), // Unwrap the cloned content
                            origin.clone(),                   // Use the cloned origin
                            slack_token.clone()               // Use the cloned Slack token
                        ).await;
                    }
                    // If the origin is None, do nothing
                    None => {}
                }
            })
        }
    });

    // This effect is triggered whenever show_edit changes value
    use_effect( move || {
        let text_copy = content_clone.to_owned();
        if show_edit() {
            // If show_edit is true, set the edited_message signal to the text of the content_clone signal
            edited_message.set(text_copy.unwrap().text.expect("No Text Found"));
            
        } 
    });

    // This effect is triggered whenever current_reaction_name changes value
    use_effect( move || {
        let reaction_name = current_reaction_name();

        if !reaction_name.is_empty() {
            // If the reaction_name is not empty, block on an async block to remove the reaction
            block_on(
                async{
                    // Acquire a lock on the user data to retrieve the Slack token
                    let user = user_lockToken_reaction.lock().await;
                    let slack_token = user.slack.user.token.clone();

                    // Clone the origin and match on its value
                    match origin_clone_reaction.clone() {
                        // If the origin is Some, proceed to remove the reaction
                        Some(origin) => {
                            // Call the remove_reaction function to remove the reaction from Slack
                            let _ = remove_reaction(
                                reaction_name, 
                                origin.clone(),
                                slack_token.clone()
                            ).await;
                        },
                        // If the origin is None, do nothing
                        None => {}
                    }

                
            })
        }
        
    });

    rsx! {
        // List item container
        li {
            // Style for the list item, using flexbox for layout
            style: format!(
                "display: flex; flex-direction: column; align-items: flex-start; margin: 10px;{}",
                if show_pane_fn() { "background-color: #3a3a3a;" } else { "" }
            ),
            // Event handler for mouse enter
            onmouseenter: move |_| {
                // Show pane if no message is currently selected
                if current_selected_id().is_none() {
                    show_pane.set(true);
                    // Set the current selected message ID to the timestamp of the origin message
                    current_selected_id.set(Some(origin_clone_mouse_enter.clone().unwrap().ts.to_string()));
                }
            },
            // Event handler for mouse leave
            onmouseleave: move |_| {
                // Hide pane if reactions or edit mode are not active
                if !show_reactions() && !show_edit() {
                    if let Some(id) = current_selected_id() {
                        // Compare the selected ID with the origin timestamp
                        let ts_id = origin_clone_mouse_leave.clone().unwrap().ts.to_string();
                        if id == ts_id {
                            show_pane.set(false);
                            current_selected_id.set(None);
                        }
                    }
                }
            },
            // Container for displaying user and timestamp
            div {
                style: "
                font-size: 0.8em; color: gray; 
                margin-top: 4px; display: flex; align-items: center;",
                // Check if sender's user field is Some
                match &sender.as_ref().unwrap().user {
                    Some(username) => rsx!(
                        span {
                            style: "
                            margin-right: 8px; font-size: 1.2em; 
                            font-weight: bold; color: white;",
                            // Display the real name of the incoming user
                            {format!("{:}",
                                incoming_user().get(username.to_string().as_str())
                                    .unwrap()
                                    .real_name
                                    .as_ref()
                                    .unwrap_or(&"".to_string())
                                    )}
                        },
                        span {
                            style: "margin-right: 4px;",
                            // Format and display the message timestamp
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
                    // Handle case where user is None
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
                // Display "(edited)" if the message has been edited
                if let Some(_) = edited {
                    div {
                        style: "
                        display: flex; flex-direction: column; 
                        align-items: flex-end; font-size: 0.8em; color: red;",
                        "(edited)"
                    }
                }
            },
            // Container for message content and media
            div{
                style: "
                display: flex; cursor: pointer; 
                justify-content: space-between; 
                align-items: center; width: 100%;",
                // Apply styling based on whether the sender is the current user
                div {
                    style: match &sender.as_ref().unwrap().user {
                        Some(user) => format!(
                            "padding: 8px; border-radius: 8px; background-color: {}; color: white; max-width: 100%;",
                            if user.to_string() == user_id {"#6CA6E1"} else {"#8A2BE2"}
                        ),
                        None => String::new(), // Return an empty string if there is no user
                    }, 
                    // Handle the message text display
                    match &content.as_ref().unwrap().text {
                        Some(text) => {
                            // Check if edit mode is active
                            if show_edit(){
                                rsx!(
                                    div{
                                        style: "display: flex;
                                                flex-direction: column;
                                                padding: 10px;
                                                background-color: #2c2f33;
                                                border-radius: 8px;
                                                box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
                                                width: 500px;
                                                max-width: 600px;
                                                margin: 20px auto;
                                                ",
                                        textarea{
                                            // Style for the textarea used for editing
                                            style: "width: 500px;
                                                    min-height: 50px;
                                                    padding: 10px;
                                                    font-size: 15px;
                                                    border: 1px solid #444;
                                                    border-radius: 4px;
                                                    resize: vertical;
                                                    outline: none;
                                                    color: #ffffff;
                                                    background-color: #23272a;
                                                    font-family: Arial, sans-serif;
                                                    ",   
                                            value: "{edited_message}",
                                            // Update edited message on input
                                            oninput: move |event| edited_message.set(event.value()),
                                        },
                                        button{
                                            // Button to send the edited message
                                            onclick: move |_| {
                                                edit_message_send.set(edited_message());
                                                show_edit.set(false); 
                                            },
                                            "Send Message"
                                        }
                                    }
                                )
                            }
                            else {
                                // Display the message text with emojis if not in edit mode
                                if ! text.is_empty() {
                                    let filtered_text = text
                                        .split(":")
                                        .map(|item| if EMOJIS.contains_key(item) { get_emoji(item) } else { item })
                                        .collect::<Vec<_>>()
                                        .join("");
            
                                    rsx!(
                                        div{
                                            style: "display: flex; align-items: center; width: 100%;",
                                            "{filtered_text}"
                                        }
                                    )
                                }
                                else {
                                    rsx!()
                                }
                            }
                        }
                       , 
                       None => {   
                                rsx!(
                                    div {
                                        style: "display: flex; align-items: center; width: 100%;",
                                        { "No Text Found".to_string() }
                                    }
                                )
                            }
                    },
                    // Image Rendering
                    for (name, (tag, download_addr)) in &img_tag_s(){
                        div{
                            style: " 
                            background-color: gray; padding: 10px; 
                            font-weight: bold; color: white; font-size: 14px;",
                            "{name}",
                            div{
                                style: "display: flex; align-items: center; gap: 10px;",
                                img{
                                    src: "{tag}",
                                    alt: "SlackImage",
                                    style: "max-height: 200px; max-width: 200px"
                                },
                                a {
                                    href: "{download_addr}",
                                    download: "{name}",
                                    style: "
                                    font-size: 24px; align-content: center; 
                                    justify-content: center;",  // Increased font size
                                    {"⬇️"}
                                }
                            }
                        }
                    },
                    // Video Rendering
                    for (name, (video, download_addr)) in &video_tag_s(){
                        div{
                            style: " 
                            background-color: gray; padding: 10px; 
                            font-weight: bold; color: white; font-size: 14px;",
                            "{name}",
                            div{
                                style: "display: flex; align-items: center; gap: 10px;",
                                video {
                                    src: "{video}",
                                    controls: true,    // Enable controls
                                    autoplay: false,   // Enable autoplay
                                    muted: true,   
                                    height: "30%", // Adjust width as needed
                                    style: "
                                    display: block; margin: 10px auto; 
                                    max-height: 200px; max-width: 200px", // Center the video if needed
                                },
                                a {
                                    href: "{download_addr}",
                                    download: "{name}",
                                    style: "font-size: 24px; align-content: center; justify-content: center;",  // Increased font size
                                    {"⬇️"}
                                }
                            }
                        }
                    },
                    // Audio Rendering
                    for (name, (audio, download_addr)) in &audio_tag_s(){
                        div{
                            style: " 
                            background-color: gray; padding: 10px; 
                            font-weight: bold; color: white; font-size: 14px;",
                            "{name}",
                            div{
                                style: "display: flex; align-items: center; gap: 10px;",
                                audio {
                                    src: "{audio}",
                                    controls: true,    // Enable controls
                                    autoplay: false,   // Set to true if you want autoplay
                                    style: "display: block; margin: 10px auto;", // Center the audio if needed
                                    // Fallback message if the audio cannot be loaded
                                    p { "Your browser does not support the audio tag." }
                                },
                                a {
                                    href: "{download_addr}",
                                    download: "{name}",
                                    style: "font-size: 24px; align-content: center; justify-content: center;",  // Increased font size
                                    {"⬇️"}
                                }
                            }
                        }
                    },
                    // Code Rendering
                    for (name, (lines, download_addr)) in &code_tag_s(){
                        div{
                            style: " 
                            background-color: gray; padding: 10px; 
                            font-weight: bold; color: white; font-size: 14px;",
                            "{name}",
                            div{
                                style: "display: flex; align-items: center; gap: 10px;",
                                div{
                                    style: "max-height: 400px;  
                                                overflow-y: scroll;
                                                background-color: #011627;
                                                color: #d6deeb; 
                                                padding: 10px; 
                                                border-radius: 8px; ",
                                    pre {
                                        code {
                                            style: " white-space: pre-wrap;
                                                        word-wrap: break-word;
                                                        font-family: monospace;
                                                    ",
                                            "{lines}"
                                        }
                                    },
                                }
                                a {
                                    href: "{download_addr}",
                                    download: "{name}",
                                    style: "font-size: 24px; align-content: center; justify-content: center;",  // Increased font size
                                    {"⬇️"}
                                }
                            }
                        }
                    },
                    // Other Files Rendering
                    for (name, download_addr) in &other_tag_s(){
                        div{
                            style: " 
                            display: flex; align-items: center; 
                            gap: 10px; background-color: gray; 
                            padding: 10px; font-weight: bold; 
                            color: white; font-size: 14px;",
                            "{name}",
                            a {
                                href: "{download_addr}",
                                download: "{name}",
                                style: "
                                font-size: 24px; align-content: center; 
                                justify-content: center;",  // Increased font size
                                {"⬇️"}
                            }
                        }
                    },
                    // Reactions display
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
                // Pane actions if the pane is shown
                if show_pane_fn() {
                    div{
                        style: "
                        display: flex; flex-direction: column; 
                        align-items: flex-end; margin-left: auto; 
                        cursor: pointer;",
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
                                onclick: move|_| show_reactions.set(!show_reactions()),
                                "➕ Add Reaction"
                            },
                            // Edit button shown only if there is no subtype and editing is allowed
                            if subtype.as_ref().is_none() && allow_edit(){
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
                                    onclick: move |_| show_edit.set(!show_edit()),
                                    "Edit Message"
                                }
                            }
                            else {
                                div{
                                    style: " 
                                    width: 90px;
                                    ", 
                                }
                            }
                        },
                        // Show emoji picker if reactions are active
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
async fn fetch_image_with_bearer(url: &str, token: &str) -> Result<String, Error> {
    // Create a new HeaderMap to store the headers for the HTTP request
    let mut headers = HeaderMap::new();

    // Insert the Authorization header with the Bearer token
    // The format is "Bearer <token>", where <token> is the provided token string
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token)).unwrap());

    // Create a new instance of the Reqwest HTTP client
    let client = ReqwestClient::new();

    // Send an asynchronous GET request to the specified URL with the headers
    // The headers contain the Authorization header with the Bearer token
    // Await the response and unwrap the result to handle any potential errors
    let response = client
        .get(url) // Specify the URL for the GET request
        .headers(headers) // Attach the headers to the request
        .send() // Send the request
        .await // Await the response
        .unwrap() // Unwrap the result to handle errors
        .bytes() // Extract the response body as bytes
        .await?; // Await the bytes and handle any errors

    // Encode the image data (bytes) into a base64 string
    // This converts the image into a format that can be easily transmitted or stored
    let base64_image = encode(response);

    // Return the base64 encoded image as a result
    Ok(base64_image)
}
/// Fetches code from a URL and returns it as a UTF-8 encoded string.
///
/// This function is used to fetch code from URLs, such as GitHub Gists or pastebins.
/// The function takes a URL and a Bearer token as input, and returns the code as a
/// UTF-8 encoded string.
///
/// The function first creates a new instance of the `reqwest::Client` and sets up
/// headers with the Bearer token. It then sends a GET request to the specified URL
/// with the Bearer token. The response is then read as bytes and converted to a
/// UTF-8 encoded string using the `String::from_utf8_lossy` method.
///
/// # Parameters
///
/// * `url`: The URL of the code to fetch
/// * `token`: The Bearer token to use for authentication
///
/// # Return
///
/// A `Result` containing the code as a UTF-8 encoded string if the request was
/// successful, or an `Error` if the request failed.
async fn fetch_code_from_url(url: &str, token: &str) -> Result<String, Error> {
    // Create a new instance of the reqwest::Client
    let client = reqwest::Client::new();

    // Set up headers with the Bearer token
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );

    // Send the GET request to the specified URL with the Bearer token
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await?;

    // Get the response body as bytes
    let body = response.bytes().await?;

    // Assuming the file is a UTF-8 encoded text file (like source code)
    // convert the bytes to a UTF-8 encoded string using the
    // String::from_utf8_lossy method
    let code = String::from_utf8_lossy(&body).to_string();

    // Return the code as a UTF-8 encoded string
    Ok(code)
}
