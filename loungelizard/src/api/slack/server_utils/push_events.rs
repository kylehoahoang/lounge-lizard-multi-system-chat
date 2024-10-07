
use std::sync::Arc;
use dioxus_logger::tracing::info;
use slack_morphism::prelude::*;

#[derive(Debug)]
struct UserStateExample(u64);

pub async fn push_events_function(
    event: SlackPushEvent,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Read state
    let current_state = {
        let states = _states.read().await;
        info!("{:#?}", states.get_user_state::<UserStateExample>());
        info!("{:#?}", states.len());
        UserStateExample(states.get_user_state::<UserStateExample>().unwrap().0 + 1)
    };

    // Write state
    {
        let mut states = _states.write().await;
        states.set_user_state::<UserStateExample>(current_state);
       info!("{:#?}", states.get_user_state::<UserStateExample>());
    }

    info!("{:#?}", event);
    Ok(())
}