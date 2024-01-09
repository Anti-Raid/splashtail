use once_cell::sync::Lazy;
use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono;
use std::fs::File;
use std::collections::HashMap;

use crate::Error;

/// Global config object
pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::load().expect("Failed to load config"));

#[derive(Serialize, Deserialize, Default)]
pub struct Differs<T: Default + Clone> {
    staging: T,
    prod: T,
}

impl<T: Default + Clone> Differs<T> {
    /// Get the value for a given environment
    pub fn get_for_env(&self, env: &str) -> T {
        if env == "staging" {
            self.staging.clone()
        } else {
            self.prod.clone()
        }
    }

    /// Get the value for the current environment
    pub fn get(&self) -> T {
        self.get_for_env(&crate::ipc::argparse::MEWLD_ARGS.current_env)
    }
}


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
    pub bot_redis_url: String,
    pub proxy: String,
    pub jobserver_url: Differs<String>,
    pub jobserver_secrets: Differs<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Sites {
    pub api: Differs<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub discord_auth: DiscordAuth,
    pub meta: Meta,
    pub sites: Sites,

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
