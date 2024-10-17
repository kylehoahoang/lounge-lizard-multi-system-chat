use serde::{Deserialize, Serialize};
use slack_morphism::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub password: String,
    pub username: String,
    pub email: String,
    pub slack: Slack,
    pub discord: Discord,
    pub ms_teams: MSTeams,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slack {
    pub app_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub config_token: String,
    pub refresh_token: String,
    pub oauth_url: String,
    pub team: Team,
    pub user: Slack_User,
    pub verif_token: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slack_User {
    pub token: String,
    pub scope: String,
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Discord {
    pub token: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MSTeams {
    pub token: String,
}


#[derive(Deserialize, Debug)]
pub struct ResponseData {
    pub ok: bool,
    pub token: String,
    pub refresh_token: String,
    pub team_id: String,
    pub user_id: String,
    pub iat: u64, // issued at time
    pub exp: u64, // expiration time
}

// Implement the Default trait for ResponseData
impl Default for ResponseData {
    fn default() -> Self {
        ResponseData {
            ok: false,
            token: String::new(),
            refresh_token: String::new(),
            team_id: String::new(),
            user_id: String::new(),
            iat: 0,
            exp: 0,
        }
    }
}

// Implement Default for User
impl Default for User {
    fn default() -> Self {
        User {
            password: String::new(),
            username: String::new(),
            email: String::new(),
            slack: Slack::default(),
            discord: Discord::default(),
            ms_teams: MSTeams::default(),
        }
    }
}

// Implement Default for Slack
impl Default for Slack {
    fn default() -> Self {
        Slack {
            app_id: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            config_token: String::new(),
            refresh_token: String::new(),
            oauth_url: String::new(),
            team: Team::default(),
            user: Slack_User::default(),
            verif_token: String::new(),
        }
    }
}

// Implement Default for Team
impl Default for Team {
    fn default() -> Self {
        Team {
            name: String::new(),
            id: String::new(),
        }
    }
}

// Implement Default for SlackUser
impl Default for Slack_User {
    fn default() -> Self {
        Slack_User {
            token: String::new(),
            scope: String::new(),
            id: String::new(),
        }
    }
}

// Implement Default for Discord
impl Default for Discord {
    fn default() -> Self {
        Discord {
            token: String::new(),
        }
    }
}

// Implement Default for MSTeams
impl Default for MSTeams {
    fn default() -> Self {
        MSTeams {
            token: String::new(),
        }
    }
}

// ! OAuth Process modified 
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ModSlackOAuthV2Response {
    pub access_token: Option<SlackApiTokenValue>,
    pub token_type: Option<SlackApiTokenType>,
    pub scope: Option<SlackApiTokenScope>,
    pub bot_user_id: Option<SlackUserId>,
    pub app_id: SlackAppId,
    pub team: SlackTeamInfo,
    pub authed_user: SlackOAuthV2AuthedUser,
    pub incoming_webhook: Option<SlackOAuthIncomingWebHook>,
}