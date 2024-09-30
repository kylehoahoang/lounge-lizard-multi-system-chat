// Synchronous function to initialize the MongoDB client
use mongodb::{sync::Client, error::Result as MongoResult, bson::doc};
use crate ::api::mongo_format::mongo_structs::*;
use dioxus_logger::tracing::{info, error, Level};

pub fn init_mongo_client() -> MongoResult<Option<Client>> {
    let mongo_str = "mongodb+srv://admin:admin@cluster0.tkwyp.mongodb.net/?retryWrites=true&w=majority";

    match Client::with_uri_str(mongo_str) {
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

pub fn add_new_user(client: &Client, user: &User) {
    let db: mongodb::sync::Database = client.database("MultisystemChat");
    let user_collection: mongodb::sync::Collection<User> = db.collection("LoungeLizard");

    let user_files = user_collection.find(doc! {"username" : "admin"}).run().unwrap();

    println!("User files: {:?}", user_files);

    // match user_collection.insert_one(*user)
    // {

    // collecting 


}

pub fn find_user(client: &Client, user: &User) {
    let db = client.database("MultisystemChat");
    let user_collection: mongodb::sync::Collection<User> = db.collection("LoungeLizard");

}

pub fn update_user(client: &Client, user: &User) {
    let db = client.database("MultisystemChat");
    let user_collection: mongodb::sync::Collection<User> = db.collection("LoungeLizard");

}
