pub mod cache;
pub mod canonical_module;
pub mod cmd;
pub mod data;
pub mod jobserver;
pub mod member_permission_calc;
pub mod module;
pub mod module_config;
pub mod punishments;
pub mod settings_poise;
pub mod sting_sources;
pub mod types;
pub mod utils;
pub mod validators;

use crate::types::{CommandExtendedData, CommandExtendedDataMap};

use std::sync::Arc;

pub type Error = Box<dyn std::error::Error + Send + Sync>; // This is constant and should be copy pasted
pub type Command = poise::Command<data::Data, Error>;
pub type Context<'a> = poise::Context<'a, data::Data, Error>;

pub struct EventHandlerContext {
    pub guild_id: serenity::all::GuildId,
    pub full_event: serenity::all::FullEvent,
    pub data: Arc<data::Data>,
    pub serenity_context: serenity::all::Context,
}

pub type BackgroundTask = (
    botox::taskman::Task,
    fn(&serenity::all::Context) -> (bool, String),
);
