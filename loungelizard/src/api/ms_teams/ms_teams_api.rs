use serde_json::{json, Value};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

/*
    Retrieve display names and ids of user's Teams

    Endpoint: https://graph.microsoft.com/v1.0/me/joinedTeams

    MS Graphs Ref: https://learn.microsoft.com/en-us/graph/api/user-list-joinedteams?view=graph-rest-1.0&tabs=http
    
    Description: Calls the endpoint to find Teams that user has joined. The display names
    can be displayed for users and the ids can be used to interact with the matching Team.
    The request also returns 'description'.

    Arguments: User's delegated access token with sufficient perms (access_token: &str)

    Returns: A json response of Team ids, display names, and more (Value)
*/
pub async fn get_teams(access_token: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let url = "https://graph.microsoft.com/v1.0/me/joinedTeams";

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        let teams: Value = response.json().await?;
        Ok(teams)
    }
    else {
        Err(format!("Failed to retrieve teams: {}", response.status()).into())
    }
}

/*
    Retrieve display names and ids of Team's Channels

    Endpoint: https://graph.microsoft.com/v1.0/teams/{team_id}/channels

    MS Graphs Ref: https://learn.microsoft.com/en-us/graph/api/channel-list?view=graph-rest-1.0&tabs=http

    Description: Calls the endpoint to find Channels that belong to the specified Team id.
    The display names can be displayed for users and the ids can be used to interact with
    the matching Channel. The request also returns 'createdDateTime', 'description',
    'membershipType', and 'isArchived'.

    Arguments: User's delegated access token with sufficient perms (access_token: &str)
    and a team id retrieved from get_teams() (team_id: &str)

    Returns: A json response of Channel ids, display names, and more (Value)
*/
pub async fn get_channels(access_token: &str, team_id: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels", team_id);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        let channels: Value = response.json().await?;
        Ok(channels)
    }
    else {
        Err(format!("Failed to retrieve channels: {}", response.status()).into())
    }
}

/*
    Retrieve id, content, and sender of all messages in a specified Channel

    Endpoint: https://graph.microsoft.com/v1.0/teams/{team_id}/channels/{channel_id}/messages

    MS Graphs Ref: https://learn.microsoft.com/en-us/graph/api/channel-list-messages?view=graph-rest-1.0&tabs=http

    Description: Calls the endpoint to find Messages that belong to the specified Channel id.
    The display names and content can be displayed for users and the ids can be used to interact
    with the matching Message. The request also returns several other attributes, too many to
    list here. If you want to see these, go to the MS Graphs Ref.

    Arguments: User's delegated access token with sufficient perms (access_token: &str),
    a team id retrieved from get_teams() (team_id: &str), and a channel id retrieved from
    get_channels() (channel_id: &str)

    Returns: A json response of Message ids, content, name of who sent it, and more (Value)
*/
pub async fn get_messages(access_token: &str, team_id: &str, channel_id: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels/{}/messages", team_id, channel_id);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        let messages: Value = response.json().await?;
        Ok(messages)
    }
    else {
        let status = response.status();
        let error_body = response.text().await?;
        eprintln!("Failed to retrieve messages: HTTP {} - {}", status, error_body);
        Err(format!("Failed to retrieve messages: HTTP {} - {}", status, error_body).into())
    }
}

/*
    Send a message using a delegated access token to a specified Channel

    Endpoint: https://graph.microsoft.com/v1.0/teams/{team_id}/channels/{channel_id}/messages

    MS Graphs Ref: https://learn.microsoft.com/en-us/graph/api/chatmessage-post?view=graph-rest-1.0&tabs=http

    Description: Calls the endpoint to send a Message to the specified Channel id.
    Currently only supports string content only. This could support a lot of
    different functionality, see MS Graphs Ref for more information.

    Arguments: User's delegated access token with sufficient perms (access_token: &str),
    a message string sent as content (message: &str), a team id retrieved from get_teams()
    (team_id: &str), and a channel id retrieved from get_channels() (channel_id: &str)

    Returns: N/A
*/
pub async fn send_message(access_token: &str, team_id: &str, channel_id: &str, message: &str,) -> Result<(), Box<dyn std::error::Error>> {

    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels/{}/messages", team_id, channel_id);

    //this can probably be modified to send different files, emojis, etc
    let body = json!({
        "body": {
            "content": message
        }
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await?;
        eprintln!("Failed to send message: HTTP {} - {}", status, error_body);
    }
    Ok(())
}