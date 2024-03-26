use moka::future::Cache;
use tokio::sync::mpsc;
use std::time::Duration;

#[derive(Clone)]
pub struct MedusaNotification {
    pub id: String,
    pub events_needed: Vec<String>,
    pub chan: mpsc::Sender<serenity::all::FullEvent>,
}

pub struct MedusaClient {
    pub notifications: Cache<String, MedusaNotification>,
}

impl MedusaClient {
    /// Creates a new medusa client
    pub fn new() -> Self {
        MedusaClient {
            notifications: Cache::builder()
            .time_to_live(Duration::from_secs(3500))
            .build(),
        }
    }

    /// Creates a new medusa notification and attaches it to a medusa client
    pub async fn new_notification(&self, events_needed: &[String], buffer_size: usize) -> (MedusaNotification, mpsc::Receiver<serenity::all::FullEvent>) {
        let (tx, rx) = mpsc::channel(buffer_size);
        
        let notification = MedusaNotification {
            id: splashcore_rs::crypto::gen_random(32),
            events_needed: events_needed.to_vec(),
            chan: tx,
        };

        self.notifications.insert(notification.id.clone(), notification.clone()).await;

        (notification, rx)
    }
}
