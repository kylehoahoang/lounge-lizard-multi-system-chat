
use serde_json::Value;

use dioxus_logger::tracing::{info, error, warn, Level};
use slack_morphism::prelude::*;

// Import the necessary error handling types  
/// Process a request from the request queue.
///
/// This function is called by `request_consumer` in an infinite loop.
/// It will process any incoming requests we have by calling `request_server`
/// with the request as an argument.
pub fn request_server(
    json_value: Value
    // TODO Intake an instance or modifiable of the UI
) 
{   
    if let Some(event_type) = json_value.get("type").and_then(|v| v.as_str()) {
        match event_type {
            // Handle event_callback events
            "event_callback" => {
                if let Some(event) = json_value.get("event") {
                    if let Ok(message_event) =
                        serde_json::from_value::<SlackMessageEvent>(event.clone())
                    {
                        // Handle subtypes of message events
                        if let Some(subtype) = message_event.subtype {
                            // Handle bot messages
                            match subtype {
                                SlackMessageEventType::BotMessage => todo!(),
                                // Handle me messages
                                SlackMessageEventType::MeMessage => todo!(),
                                // Handle channel join events
                                SlackMessageEventType::ChannelJoin => todo!(),
                                // Handle channel leave events
                                SlackMessageEventType::ChannelLeave => todo!(),
                                // Handle bot add events
                                SlackMessageEventType::BotAdd => todo!(),
                                // Handle bot remove events
                                SlackMessageEventType::BotRemove => todo!(),
                                // Handle channel topic change events
                                SlackMessageEventType::ChannelTopic => todo!(),
                                // Handle channel purpose change events
                                SlackMessageEventType::ChannelPurpose => todo!(),
                                // Handle channel name change events
                                SlackMessageEventType::ChannelName => todo!(),
                                // Handle file share events
                                SlackMessageEventType::FileShare => todo!(),
                                // Handle message change events
                                SlackMessageEventType::MessageChanged => todo!(),
                                // Handle message delete events
                                SlackMessageEventType::MessageDeleted => todo!(),
                                // Handle thread broadcast events
                                SlackMessageEventType::ThreadBroadcast => todo!(),
                                // Handle tombstone events
                                SlackMessageEventType::Tombstone => todo!(),
                                // Handle joiner notification events
                                SlackMessageEventType::JoinerNotification => todo!(),
                                // Handle slackbot response events
                                SlackMessageEventType::SlackbotResponse => todo!(),
                                // Handle emoji change events
                                SlackMessageEventType::EmojiChanged => todo!(),
                                // Handle huddle room created events
                                SlackMessageEventType::SlackHuddleRoomCreated => todo!(),
                                // Handle channel archive events
                                SlackMessageEventType::ChannelArchive => todo!(),
                                // Handle channel unarchive events
                                SlackMessageEventType::ChannelUnarchive => todo!(),
                                // Handle app conversation leave events
                                SlackMessageEventType::AppConversationLeave => todo!(),
                                // Handle bot enable events
                                SlackMessageEventType::BotEnable => todo!(),
                                // Handle bot disable events
                                SlackMessageEventType::BotDisable => todo!(),
                                // Handle pinned item events
                                SlackMessageEventType::PinnedItem => todo!(),
                                // Handle reminder add events
                                SlackMessageEventType::ReminderAdd => todo!(),
                                // Handle file comment events
                                SlackMessageEventType::FileComment => todo!(),
                                // Handle file created events
                                SlackMessageEventType::FileCreated => todo!(),
                                // Handle file changed events
                                SlackMessageEventType::FileChanged => todo!(),
                                // Handle file deleted events
                                SlackMessageEventType::FileDeleted => todo!(),
                                // Handle file shared events
                                SlackMessageEventType::FileShared => todo!(),
                                // Handle file unshared events
                                SlackMessageEventType::FileUnshared => todo!(),
                                // Handle file public events
                                SlackMessageEventType::FilePublic => todo!(),
                                _ => {
                                    println!("Unknown subtype");
                                }
                            }
                        } else {
                            // Handle regular messages
                            println!("Regular message")
                        }
                    }

                    else if let Some(reaction_type) = event.get("type").and_then(|v| v.as_str()) {
                        match reaction_type {
                            "reaction_added" => {
                                if let Ok(_event_t) =
                                    serde_json::from_value::<SlackReactionAddedEvent>(event.clone())
                                {
                                    // Handle reaction added events
                                }
                            }
                            "reaction_removed" => {
                                if let Ok(_event_t) =
                                    serde_json::from_value::<SlackReactionRemovedEvent>(event.clone())
                                {
                                    // Handle reaction removed events
                                }
                            }
                            _ => {
                                warn!("Unknown reaction type: {}", reaction_type);
                            }
                        }
                    }
                    else {
                        info!("{:#?}", event);
                    }
                }
            }
            _ => {
            }
        }
    }

    
}