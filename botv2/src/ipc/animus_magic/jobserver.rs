use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum JobserverAnimusResponse {
    /// Probe response
    Probe {
        message: String
    },

    /// SpawnTask response
    SpawnTask {
        task_id: String
    }
}

#[derive(Serialize, Deserialize)]
pub enum JobserverAnimusMessage {
    /// Probe operation
    Probe {},

    /// Spawn task. Note that oneof create and execute must be true
    SpawnTask {
        name: String,
        data: serde_json::Value,
        create: bool,
        execute: bool,
        task_id: Option<String> // If create is false, this is required
    }
}
