use once_cell::sync::Lazy;
use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono;
use std::fs::File;

use crate::Error;

/// Global config object
pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::load().expect("Failed to load config"));

#[derive(Serialize, Deserialize, Default)]
pub struct DiscordAuth {
    pub token: String,
    pub client_id: String,
    pub client_secret: String,
    pub can_use_bot: Vec<UserId>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Meta {
    pub web_redis_channel: String,
    pub postgres_url: String,
    pub proxy: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub discord_auth: DiscordAuth,
    pub meta: Meta,

    #[serde(skip)]
    /// Setup by load() for statistics
    pub bot_start_time: i64, 
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        // Open config.yaml from parent directory
        let file = File::open("../config.yaml");

        match file {
            Ok(file) => {
                // Parse config.yaml
                let mut cfg: Config = serde_yaml::from_reader(file)?;

                cfg.bot_start_time = chrono::Utc::now().timestamp();

                // Return config
                Ok(cfg)
            }
            Err(e) => {
                // Print error
                println!("config.yaml could not be loaded: {}", e);

                // Exit
                std::process::exit(1);
            }
        }
    }
}
