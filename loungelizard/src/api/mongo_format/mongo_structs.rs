use serde::{Deserialize, Serialize};

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
    pub bot: Bot,
    pub client_id: String,
    pub client_secret: String,
    pub config_token: String,
    pub oauth_url: String,
    pub team: Team,
    pub user: Slack_User,
    pub verif_token: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bot {
    pub token: String,
    pub scope: String,
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
    pub access_token: String,
    pub refresh_token: String,
    pub expiration: String,
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
            bot: Bot::default(),
            client_id: String::new(),
            client_secret: String::new(),
            config_token: String::new(),
            oauth_url: String::new(),
            team: Team::default(),
            user: Slack_User::default(),
            verif_token: String::new(),
        }
    }
}

// Implement Default for Bot
impl Default for Bot {
    fn default() -> Self {
        Bot {
            token: String::new(),
            scope: String::new(),
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
            access_token: String::new(),
            refresh_token: String::new(),
            expiration: String::new()
        }
    }
}
