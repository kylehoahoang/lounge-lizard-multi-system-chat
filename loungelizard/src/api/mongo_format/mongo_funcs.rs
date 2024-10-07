// Synchronous function to initialize the MongoDB client
use mongodb::{sync::Client, error::Result as MongoResult};
use dioxus_logger::tracing::{info, error};


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

