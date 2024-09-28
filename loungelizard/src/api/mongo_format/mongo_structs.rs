use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    password: String,
    pub username: String,
    email: String,
    slack: Slack,
    discord: Discord,
    ms_teams: MSTeams,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Slack {
    app_id: String,
    bot: Bot,
    client_id: String,
    client_secret: String,
    config_token: String,
    oauth_url: String,
    redirect_host: String,
    team: Team,
    user: SlackUser,
    verif_token: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bot {
    token: String,
    scope: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Team {
    name: String,
    id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlackUser {
    token: String,
    scope: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Discord {
    token: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MSTeams {
    token: String,
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
            redirect_host: String::new(),
            team: Team::default(),
            user: SlackUser::default(),
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
impl Default for SlackUser {
    fn default() -> Self {
        SlackUser {
            token: String::new(),
            scope: String::new(),
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
