// Synchronous function to initialize the MongoDB client
use mongodb::{sync::Client, bson::doc};
use dioxus_logger::tracing::{info, error, Level};

fn init_mongo_client() -> Result<Client, mongodb::error::Error> {
    let mongo_str = "mongodb+srv://admin:admin@cluster0.tkwyp.mongodb.net/?retryWrites=true&w=majority";

    match Client::with_uri_str(mongo_str) {
        Ok(client) => {
            info!("MongoDB connected successfully");
            Ok(client)
        }
        Err(e) => {
            error!("MongoDB connection failed: {}", e);
            Err(e)
        }
    }
}