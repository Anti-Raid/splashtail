use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum JobserverAnimusResponse {
    /// SpawnTask response
    SpawnTask { task_id: String },
}

#[derive(Serialize, Deserialize)]
pub enum JobserverAnimusMessage {
    /// Spawn task. Note that oneof create and execute must be true
    SpawnTask {
        name: String,
        data: serde_json::Value,
        create: bool,
        execute: bool,
        task_id: Option<String>, // If create is false, this is required
        user_id: String,
    },
}
