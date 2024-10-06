pub mod ar_event;
pub mod cache;
pub mod canonical_module;
pub mod cmd;
pub mod data;
pub mod jobserver;
pub mod member_permission_calc;
pub mod module;
pub mod module_config;
pub mod punishments;
pub mod settings_autogen;
pub(crate) mod settings_poise; // Only used by settings_autogen
pub mod stings;
pub mod types;
pub mod utils;
pub mod validators;

use crate::types::{CommandExtendedData, CommandExtendedDataMap};

pub type Error = Box<dyn std::error::Error + Send + Sync>; // This is constant and should be copy pasted
pub type Command = poise::Command<data::Data, Error>;
pub type Context<'a> = poise::Context<'a, data::Data, Error>;

pub type BackgroundTask = (
    botox::taskman::Task,
    fn(&serenity::all::Context) -> (bool, String),
);
