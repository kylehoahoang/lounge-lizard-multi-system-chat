
use std::sync::Arc;
use ::log::{debug, error, info};
use slack_morphism::prelude::*;


pub fn error_handler(
    err: Box<dyn std::error::Error + Send + Sync>,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> HttpStatusCode {
    error!("{:#?}", err);

    // Defines what we return Slack server
    HttpStatusCode::BAD_REQUEST
}