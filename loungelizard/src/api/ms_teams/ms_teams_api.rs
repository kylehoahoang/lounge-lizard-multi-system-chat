
use hyper::client;
use serde_json::{json, Value};
use std::error::Error;
use std::collections::{HashMap, HashSet};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use futures::stream::{self, StreamExt};
use chrono::{DateTime, Duration, ParseError, Utc};
use ammonia::Builder;
use regex::Regex;

type UserCache = HashMap<String, (String, String)>; // user_id -> (displayName, profilePicture)

/*
    Check token validity for API usage.

    Description: Check the expiration of a token to the current
    time. If it is invalid, request a new one using a refresh
    token. Otherwise, continue as normal. This should be used
    before every API call is made.

    Arguments: A client id (client_id: &str), access token
    (access_token: &str), refresh token (refresh_token: &str),
    and a time that the access token expires (expiration: &str)

    Returns: Returns if the access token is valid (bool)
*/
async fn ensure_valid_token(client_id: &str, access_token: &str, refresh_token: &str, expiration: &str) -> Result<(bool, String, String, String), Box<dyn Error>> {
    let expiration_time: chrono::DateTime<Utc> = chrono::DateTime::parse_from_rfc3339(expiration)
        .map_err(|e: chrono::ParseError| format!("Failed to parse expiration time: {}", e))?
        .with_timezone(&Utc);

    if Utc::now() >= expiration_time {
        let (new_access_token, new_refresh_token, new_expiration) = refresh_access_token(client_id, refresh_token).await?;
        Ok((false, new_access_token, new_refresh_token, new_expiration))
    }
    else {
        Ok((true, "".to_string(), "".to_string(), "".to_string()))
    }
}

/*
    Refresh an invalid access token.

    Endpoint: https://login.microsoft.com/organizations/oauth2/token

    Description: This function requests a new access token for
    further use with the api. The request is much simpler, as
    it only requires the refresh token and a client id.

    Arguments: The client id (client_id: &str) and a refresh token
    (refresh_token: &str)

    Returns: Returns a refreshed/valid access token (String)
*/
async fn refresh_access_token(client_id: &str, refresh_token: &str) -> Result<(String, String, String), Box<dyn Error>> {
    let token_url = "https://login.microsoft.com/organizations/oauth2/token";

    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("grant_type", "refresh_token");
    params.insert("refresh_token", refresh_token);

    let client = reqwest::Client::new();
    let response = client
        .post(token_url)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await?;
        return Err(format!("Failed to get access token: HTTP {} - {}", status, error_body).into());
    }

    let token_response: HashMap<String, String> = response.json().await?;
    let access_token = token_response.get("access_token").ok_or("Access token not found")?.clone();
    let refresh_token = token_response.get("refresh_token").ok_or("Refresh token not found")?.clone();
    
    let expires_in: u64 = token_response
        .get("expires_in")
        .and_then(|s| s.parse().ok())
        .unwrap_or(3600);
    let expiration_time = Utc::now() + chrono::Duration::seconds(expires_in as i64);
    let expiration = expiration_time.to_rfc3339();

    Ok((access_token, refresh_token, expiration))
}

pub async fn get_user(access_token: &str) -> Result<Value, Box<dyn Error>> {
    let url = "https://graph.microsoft.com/v1.0/me";

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await?;

    if response.status().is_success() {
        let user: Value = response.json().await?;
        let id = user.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string();
        let display_name = user.get("displayName").and_then(|s| s.as_str()).unwrap_or("").to_string();
    
        Ok(json!({
            "id": id,
            "displayName": display_name
        }))

    }
    else {
        Err(format!("Failed to retrieve user: {}", response.status()).into())
    }
}

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
pub async fn get_teams(access_token: &str) -> Result<Value, Box<dyn Error>> {
    let url = "https://graph.microsoft.com/v1.0/me/joinedTeams";

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        let wrapped_teams: Value = response.json().await?;
        if let Some(teams) = wrapped_teams.get("value").and_then(|val| val.as_array()) {
            let mut parsed_teams = Vec::new();
            for team in teams {
                let id = team.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string();
                let display_name = team.get("displayName").and_then(|s| s.as_str()).unwrap_or("").to_string();
                let team_picture = get_team_picture(access_token, &id).await?;
                parsed_teams.push(json!({
                    "id": id,
                    "displayName": display_name,
                    "teamPicture": team_picture
                }));
            }
            Ok(json!(parsed_teams))
        }
        else {
            Err("Response does not contain 'value' field".into())
        }
    }
    else {
        Err(format!("Failed to retrieve teams: {}", response.status()).into())
    }
}

pub async fn get_users(access_token: &str, team_id: &str) -> Result<UserCache, Box<dyn Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/members", team_id);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        let members: Value = response.json().await?;
        let mut user_cache: UserCache = HashMap::new();

        if let Some(users) = members.get("value").and_then(|v| v.as_array()) {
            for user in users {
                if let Some(user_id) = user.get("userId").and_then(|id| id.as_str()) {
                    let display_name = user.get("displayName").and_then(|dn| dn.as_str()).unwrap_or("Unknown User").to_string();
                    let profile_picture = get_user_picture(access_token, user_id).await.unwrap_or("".to_string());
                    user_cache.insert(user_id.to_string(), (display_name.clone(), profile_picture.clone()));
                }
            }
        }
        Ok(user_cache)
    }
    else {
        let status = response.status();
        let error_body = response.text().await?;
        Err(format!("Failed to retrieve users: HTTP {} - {}", status, error_body).into())
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
pub async fn get_channels(access_token: &str, team_id: &str) -> Result<Value, Box<dyn Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels", team_id);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    if response.status().is_success() {
        let wrapped_channels: Value = response.json().await?;
        if let Some(channels) = wrapped_channels.get("value").and_then(|val| val.as_array()) {
            let mut parsed_channels = Vec::new();
            for channel in channels {
                let id = channel.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string();
                let display_name = channel.get("displayName").and_then(|s| s.as_str()).unwrap_or("").to_string();
                parsed_channels.push(json!({
                    "id": id,
                    "displayName": display_name
                }));
            }
            Ok(json!(parsed_channels))
        }
        else {
            Err("Response does not contain 'value' field".into())
        }
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
pub async fn get_messages(access_token: &str, team_id: &str, channel_id: &str, user_cache: &UserCache) -> Result<Value, Box<dyn Error>> {
    let mut all_messages = Vec::new();
    let mut url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels/{}/messages?$expand=replies", team_id, channel_id);

    let client = reqwest::Client::new();

    loop {
        let response = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;
    
        if response.status().is_success() {
            let wrapped_messages: Value = response.json().await?;
            if let Some(messages) = wrapped_messages.get("value").and_then(|val| val.as_array()) {
                for message in messages {
                    let parsed_message = parse_message(message, user_cache);
                    if let Some(user) = parsed_message.get("user") {
                        if user.get("displayName").is_none() || user.get("displayName") == Some(&Value::String("Unknown User".to_string())) {
                            continue;
                        }
                        all_messages.push(parsed_message);
                    }
                }
            }
            if let Some(next_link) = wrapped_messages.get("@odata.nextLink").and_then(|link| link.as_str()) {
                url = next_link.to_string();
            }
            else {
                break;
            }
        }
    }
    
    sort_messages_by_time(&mut all_messages);
    Ok(Value::Array(all_messages))
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
pub async fn send_message(access_token: &str, team_id: &str, channel_id: &str, message: &str, subject: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels/{}/messages", team_id, channel_id);

    let body = json!({
        "subject": subject,
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

pub async fn send_message_reply(access_token: &str, team_id: &str, channel_id: &str, message_id: &str, message: &str,) -> Result<(), Box<dyn Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels/{}/messages/{}/replies", team_id, channel_id, message_id);

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
        eprintln!("Failed to send reply: HTTP {} - {}", status, error_body);
    }
    Ok(())
}

pub async fn send_reaction(access_token: &str, team_id: &str, channel_id: &str, message_id: &str, reaction: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels/{}/messages/{}/setReaction", team_id, channel_id, message_id);

    let payload = serde_json::json!({
        "reactionType": reaction
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .json(&payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(())
    }
    else {
        let status = response.status();
        let error_body = response.text().await?;
        Err(format!("Failed to add reaction: HTTP {} - {}", status, error_body).into())
    }
}

pub async fn send_reaction_reply(access_token: &str, team_id: &str, channel_id: &str, message_id: &str, reply_id: &str, reaction: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels/{}/messages/{}/replies/{}/setReaction", team_id, channel_id, message_id, reply_id);

    let payload = serde_json::json!({
        "reactionType": reaction
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .json(&payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(())
    }
    else {
        let status = response.status();
        let error_body = response.text().await?;
        Err(format!("Failed to add reaction: HTTP {} - {}", status, error_body).into())
    }
}

pub async fn remove_reaction(access_token: &str, team_id: &str, channel_id: &str, message_id: &str, reaction: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels/{}/messages/{}/unsetReaction", team_id, channel_id, message_id);

    let payload = serde_json::json!({
        "reactionType": reaction
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .json(&payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(())
    }
    else {
        let status = response.status();
        let error_body = response.text().await?;
        Err(format!("Failed to add reaction: HTTP {} - {}", status, error_body).into())
    }
}

pub async fn remove_reaction_reply(access_token: &str, team_id: &str, channel_id: &str, message_id: &str, reply_id: &str, reaction: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/channels/{}/messages/{}/replies/{}/unsetReaction", team_id, channel_id, message_id, reply_id);

    let payload = serde_json::json!({
        "reactionType": reaction
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .json(&payload)
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(())
    }
    else {
        let status = response.status();
        let error_body = response.text().await?;
        Err(format!("Failed to add reaction: HTTP {} - {}", status, error_body).into())
    }
}

pub async fn get_team_picture(access_token: &str, team_id: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/teams/{}/photo/$value", team_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;
        let base64_image = base64::encode(bytes);
        Ok(format!("data:image/jpeg;base64,{}", base64_image))
    }
    else {
        Err(format!("Failed to retrieve team picture: HTTP {}", response.status()).into())
    }
}

pub async fn get_user_picture(access_token: &str, user_id: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("https://graph.microsoft.com/v1.0/users/{}/photo/$value", user_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .send()
        .await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;
        let base64_image = base64::encode(bytes);
        Ok(format!("data:image/jpeg;base64,{}", base64_image))
    }
    else {
        Err(format!("Failed to retrieve profile picture: HTTP {}", response.status()).into())
    }
}

pub fn sanitize_message(content: &str) -> String {
    let emoji_re = Regex::new(r#"<emoji[^>]*alt="([^"]+)"[^>]*></emoji>"#).unwrap();

    let with_emojis = emoji_re.replace_all(content, "$1");

    let allowed_tags: HashSet<&str> = HashSet::new();
    
    let sanitized_content = Builder::new()
        .tags(allowed_tags)
        .clean(&with_emojis)
        .to_string();

    let space_re = Regex::new(r"&nbsp;|\\s+").unwrap();
    let final_content = space_re.replace_all(&sanitized_content, " ").trim().to_string();

    final_content
}

fn get_reaction_emoji(reaction_type: &str) -> String {
    let reaction_map: HashMap<&str, &str> = [
        ("like", "👍"),
        ("heart", "❤️"),
        ("laugh", "😆"),
        ("surprised", "😮"),
        ("sad", "😢"),
        ("angry", "😠"),
        ("😁", "😁"),
        ("😡", "😡"),
        ("🤣", "🤣"),
        ("😂", "😂"),
        ("😍", "😍"),
        ("😢", "😢")
    ]
    .iter()
    .cloned()
    .collect();

    reaction_map.get(reaction_type).map_or("❓".to_string(), |&v| v.to_string()) // Use a default if the reactionType is unknown
}

pub fn parse_message(message: &Value, user_cache: &UserCache) -> Value {
    let subject = message.get("subject").and_then(|s| s.as_str()).unwrap_or("").to_string();
    let content = message.get("body").and_then(|b| b.get("content")).and_then(|c| c.as_str()).map(|c| sanitize_message(c)).unwrap_or("".to_string());
    let time = message.get("createdDateTime").and_then(|dt| dt.as_str()).unwrap_or("").to_string();
    let user = message.get("from").map(|u| parse_user(u, user_cache));
    let reactions = parse_reactions(message.get("reactions").unwrap_or(&json!([])), user_cache);
    let replies = message.get("replies").and_then(|r| r.as_array()).unwrap_or(&Vec::new()).iter().map(|reply| parse_reply(reply, user_cache)).collect::<Vec<_>>();

    json!({
        "id": message.get("id"),
        "subject": subject,
        "content": content,
        "time": time,
        "user": user.unwrap_or(json!({})),
        "reactions": reactions,
        "replies": replies
    })
}

fn parse_user(user: &Value, user_cache: &UserCache) -> Value {
    if let Some(user) = user.get("user") {
        let id = user.get("id").and_then(|id| id.as_str()).unwrap_or("").to_string();
        if let Some((display_name, profile_picture)) = user_cache.get(&id) {
            json!({
                "id": id,
                "displayName": display_name,
                "profilePicture": profile_picture
            })
        }
        else {
            json!({
                "id": id,
                "displayName": "Unknown User",
                "profilePicture": ""
            })
        }
    }
    else {
        json!({
            "id": "Unknown ID",
            "displayName": "Unknown User",
            "profilePicture": ""
        })
    }
}

fn parse_reply(reply: &Value, user_cache: &UserCache) -> Value {
    let reply_from_id = reply.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let content = reply.get("body").and_then(|b| b.get("content")).and_then(|c| c.as_str()).map(|c| sanitize_message(c)).unwrap_or("".to_string());
    let time = reply.get("createdDateTime").and_then(|dt| dt.as_str()).unwrap_or("").to_string();
    let user = reply.get("from").map(|u| parse_user(u, user_cache));
    let reply_reactions = parse_reactions(reply.get("reactions").unwrap_or(&json!([])), user_cache);

    json!({
        "id": reply.get("id"),
        "replyFromId": reply_from_id,
        "time": time,
        "content": content,
        "user": user.unwrap_or(json!({})),
        "reactions": reply_reactions
    })
}

fn parse_reactions(reactions: &Value, user_cache: &UserCache) -> Vec<Value> {
    reactions.as_array().unwrap_or(&Vec::new()).iter().map(|reaction| {
        let reaction_type = reaction.get("reactionType").and_then(|rt| rt.as_str()).unwrap_or("").to_string();
        let emoji = get_reaction_emoji(&reaction_type);
        let user = reaction.get("user").map(|u| parse_user(u, user_cache));
        json!({
            "reactionType": reaction_type,
            "emoji": emoji,
            "user": user.unwrap_or(json!({}))
        })
    }).collect()
}
fn sort_messages_by_time(messages: &mut Vec<Value>) {
    messages.sort_by(|a, b| {
        let time_a = a.get("time").and_then(|t| t.as_str()).unwrap_or("");
        let time_b = b.get("time").and_then(|t| t.as_str()).unwrap_or("");
        time_a.cmp(time_b)
    });
}
