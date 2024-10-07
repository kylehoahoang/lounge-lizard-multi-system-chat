use slack_morphism::prelude::*;
use std::sync::Arc;
use dioxus_logger::tracing::info;


pub async fn interaction_events_function(
    event: SlackInteractionEvent,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Interevent {:#?}", event);
    Ok(())
}