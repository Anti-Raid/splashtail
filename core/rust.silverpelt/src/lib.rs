pub mod ar_event;
pub mod cache;
pub mod canonical_module;
pub mod data;
pub mod member_permission_calc;
pub mod module;
pub mod module_config;
pub mod punishments;
pub mod stings;
pub mod types;
pub mod utils;

use crate::types::{CommandExtendedData, CommandExtendedDataMap};

pub type Error = Box<dyn std::error::Error + Send + Sync>; // This is constant and should be copy pasted
pub type Command = poise::Command<data::Data, Error>;
pub type Context<'a> = poise::Context<'a, data::Data, Error>;

pub type BackgroundTask = (
    botox::taskman::Task,
    fn(&serenity::all::Context) -> (bool, String),
);
