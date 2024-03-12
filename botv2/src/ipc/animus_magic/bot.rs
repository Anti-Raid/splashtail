use crate::impls::cache::CacheHttpImpl;
/// Bot animus contains the request and response for a bot
///
/// To edit/add responses, add them both to bot.rs and to splashcore/animusmagic/types.go
use crate::silverpelt::{
    canonical_module::CanonicalModule,
    silverpelt_cache::SILVERPELT_CACHE,
};
use crate::Error;
use serde::{Deserialize, Serialize};
use serenity::all::{GuildId, Role, RoleId, UserId};

#[derive(Serialize, Deserialize)]
pub enum BotAnimusResponse {
    /// Modules event contains module related data
    Modules { modules: Vec<CanonicalModule> },
    /// GuildsExist event contains a list of u8s, where 1 means the guild exists and 0 means it doesn't
    GuildsExist { guilds_exist: Vec<u8> },
    /// BaseGuildUserInfo event is described in AnimusMessage
    BaseGuildUserInfo {
        owner_id: String,
        name: String,
        icon: Option<String>,
        /// List of all roles in the server
        roles: std::collections::HashMap<RoleId, Role>,
        /// List of roles the user has
        user_roles: Vec<RoleId>,
        /// List of roles the bot has
        bot_roles: Vec<RoleId>,
    },
}

#[derive(Serialize, Deserialize)]
pub enum BotAnimusMessage {
    /// Ask the bot for module data
    Modules {},
    /// Given a list of guild IDs, return whether or not they exist on the bot
    GuildsExist { guilds: Vec<GuildId> },
    /// Given a guild ID and a user ID, check:
    /// - The server owner
    /// - The server name
    /// - The server icon
    /// - The users roles
    /// - The bots highest role
    BaseGuildUserInfo { guild_id: GuildId, user_id: UserId },
}

impl BotAnimusMessage {
    pub async fn response(&self, cache_http: &CacheHttpImpl) -> Result<BotAnimusResponse, Error> {
        match self {
            Self::Modules {} => {
                let mut modules = Vec::new();

                for idm in SILVERPELT_CACHE.canonical_module_cache.iter() {
                    let module = idm.value();
                    modules.push(module.clone());
                }

                Ok(BotAnimusResponse::Modules { modules })
            }
            Self::GuildsExist { guilds } => {
                let mut guilds_exist = Vec::with_capacity(guilds.len());

                for guild in guilds {
                    guilds_exist.push({
                        if cache_http.cache.guild(*guild).is_some() {
                            1
                        } else {
                            0
                        }
                    });
                }

                Ok(BotAnimusResponse::GuildsExist { guilds_exist })
            }
            Self::BaseGuildUserInfo { guild_id, user_id } => {
                let (name, icon, owner, roles, user_roles, bot_roles) = {
                    let guild = match cache_http.cache.guild(*guild_id) {
                        Some(guild) => guild,
                        None => return Err("Guild not found".into()),
                    }
                    .clone();

                    let user_roles = {
                        let mem = match guild.member(cache_http, *user_id).await {
                            Ok(member) => member,
                            Err(e) => return Err(format!("Failed to get member: {}", e).into()),
                        };

                        mem.roles.to_vec()
                    };

                    let bot_user_id = cache_http.cache.current_user().id;
                    let bot_roles = guild.member(&cache_http, bot_user_id).await?.roles.to_vec();

                    (
                        guild.name.to_string(),
                        guild.icon_url(),
                        guild.owner_id,
                        guild.roles,
                        user_roles,
                        bot_roles,
                    )
                };

                Ok(BotAnimusResponse::BaseGuildUserInfo {
                    name,
                    icon,
                    owner_id: owner.to_string(),
                    roles,
                    user_roles,
                    bot_roles,
                })
            },
        }
    }
}
