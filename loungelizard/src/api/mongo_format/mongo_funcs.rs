// Synchronous function to initialize the MongoDB client
use mongodb::{sync::Client, error::Result as MongoResult, bson::doc};
use dioxus_logger::tracing::{info, error, warn};
use crate::api::mongo_format::mongo_structs::*;
use bson::to_bson;
use std::sync::Arc;
use tokio::sync::Mutex;


const MONGO_CLUSTER: &'static str = "mongodb+srv://admin:admin@cluster0.tkwyp.mongodb.net/?retryWrites=true&w=majority";
pub const MONGO_DATABASE: &'static str = "MultisystemChat";
pub const MONGO_COLLECTION: &'static str = "LoungeLizard";


pub fn init_mongo_client() -> MongoResult<Option<Client>> {
    
    match Client::with_uri_str(MONGO_CLUSTER) {
        Ok(client) => {
            info!("MongoDB connected successfully");
            Ok(Some(client))
        }
        Err(e) => {
            error!("MongoDB connection failed: {}", e);
            Ok(None)
        }
    }
}

pub async fn update_slack(
    user: User,
    mongo_client: Arc<Mutex<Option<Client>>>,
)
{
    let mongo_lock = mongo_client.lock().await;

    if let Some(client) = mongo_lock.as_ref(){
        let client_clone = client.clone();
        let user_clone = user.clone();

        let db = client_clone.database(MONGO_DATABASE);
        let user_collection = db.collection::<User>(MONGO_COLLECTION);
        
        match to_bson(&user_clone.slack) {
            Ok(slack_bson) => {
                match user_collection
                    .find_one_and_update(
                        doc! { 
                            "$or": [{"username": &user_clone.username}, 
                                    {"email": &user_clone.email}] 
                        },
                        doc! {
                            "$set": { "slack" : slack_bson }
                        }
                    )
                    .await 
                {
                    Ok(Some(_)) => {
                        // Document found and updated
                        info!("User Document Updated Successfully for Slack");
                    }
                    Ok(None) => {
                        // No document matched the filter
                        warn!("Document not found");
                    }
                    Err(e) => {
                        error!("Something went wrong: {:#?}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to convert Slack to BSON: {:#?}", e);
            }
        }
    }
    else {
        error!("No Mongo Client Available")
    }

}

pub async fn update_discord(
    user: User,
    mongo_client: Arc<Mutex<Option<Client>>>,
)
{
    let mongo_lock = mongo_client.lock().await;

    if let Some(client) = mongo_lock.as_ref(){
        let client_clone = client.clone();
        let user_clone = user.clone();

        let db = client_clone.database(MONGO_DATABASE);
        let user_collection = db.collection::<User>(MONGO_COLLECTION);
        
        match to_bson(&user_clone.discord) {
            Ok(discord_bson) => {
                match user_collection
                    .find_one_and_update(
                        doc! { 
                            "$or": [{"username": &user_clone.username}, 
                                    {"email": &user_clone.email}] 
                        },
                        doc! {
                            "$set": { "discord" : discord_bson }
                        }
                    )
                    .await 
                {
                    Ok(Some(_)) => {
                        // Document found and updated
                        info!("User Document Updated Successfully for Discord");
                    }
                    Ok(None) => {
                        // No document matched the filter
                        warn!("Document not found");
                    }
                    Err(e) => {
                        error!("Something went wrong: {:#?}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to convert Discord to BSON: {:#?}", e);
            }
        }
    }
    else {
        error!("No Mongo Client Available")
    }

}

pub async fn update_ms_teams(
    user: User,
    mongo_client: Arc<Mutex<Option<Client>>>,
)
{
    let mongo_lock = mongo_client.lock().await;

    if let Some(client) = mongo_lock.as_ref(){
        let client_clone = client.clone();
        let user_clone = user.clone();

        let db = client_clone.database(MONGO_DATABASE);
        let user_collection = db.collection::<User>(MONGO_COLLECTION);
        
        match to_bson(&user_clone.ms_teams) {
            Ok(teams_bson) => {
                match user_collection
                    .find_one_and_update(
                        doc! { 
                            "$or": [{"username": &user_clone.username}, 
                                    {"email": &user_clone.email}] 
                        },
                        doc! {
                            "$set": { "ms_teams" : teams_bson }
                        }
                    )
                    .await 
                {
                    Ok(Some(_)) => {
                        // Document found and updated
                        info!("User Document Updated Successfully for MSTeams");
                    }
                    Ok(None) => {
                        // No document matched the filter
                        warn!("Document not found");
                    }
                    Err(e) => {
                        error!("Something went wrong: {:#?}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to convert MSTeams to BSON: {:#?}", e);
            }
        }
    }
    else {
        error!("No Mongo Client Available")
    }

}

