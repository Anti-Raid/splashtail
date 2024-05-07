use serde::{Deserialize, Serialize};
use splashcore_rs::animusmagic::client::{
    AnimusMessage, AnimusResponse, SerializableAnimusMessage, SerializableAnimusResponse,
};
use splashcore_rs::animusmagic::protocol::AnimusTarget;

#[derive(Serialize, Deserialize)]
pub enum JobserverAnimusResponse {
    /// SpawnTask response
    SpawnTask { task_id: String },
}

impl AnimusResponse for JobserverAnimusResponse {
    fn target(&self) -> AnimusTarget {
        AnimusTarget::Bot
    }
}
impl SerializableAnimusResponse for JobserverAnimusResponse {}

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

impl AnimusMessage for JobserverAnimusMessage {
    fn target(&self) -> AnimusTarget {
        AnimusTarget::Bot
    }
}
impl SerializableAnimusMessage for JobserverAnimusMessage {}
