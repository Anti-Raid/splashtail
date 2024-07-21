/// Bot animus contains the request and response for a bot
///
/// To edit/add responses, add them both to bot.rs and to go.std/animusmagic/types.go
use modules::silverpelt::canonical_module::CanonicalModule;
use modules::silverpelt::silverpelt_cache::SILVERPELT_CACHE;
use splashcore_rs::types::silverpelt::PermissionResult;

use splashcore_rs::animusmagic::client::{
    AnimusMessage, AnimusResponse, SerializableAnimusMessage, SerializableAnimusResponse,
};
use splashcore_rs::animusmagic::protocol::{AnimusErrorResponse, AnimusTarget};

use module_settings::{self, canonical_types::CanonicalSettingsError, types::OperationType};
use serde::{Deserialize, Serialize};
use serenity::all::{GuildChannel, GuildId, Permissions, Role, RoleId, UserId};
use splashcore_rs::value::Value;
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]

pub enum CanonicalSettingsResult {
    Ok {
        fields: Vec<indexmap::IndexMap<String, serde_json::Value>>,
    },
    PermissionError {
        res: splashcore_rs::types::silverpelt::PermissionResult,
    },
    Err {
        error: CanonicalSettingsError,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GuildChannelWithPermissions {
    pub user: Permissions,
    pub bot: Permissions,
    pub channel: GuildChannel,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum BotAnimusResponse {
    Ok {
        message: String,
    },
    /// Modules event contains module related data
    Modules {
        modules: Vec<CanonicalModule>,
    },
    /// GuildsExist event contains a list of u8s, where 1 means the guild exists and 0 means it doesn't
    GuildsExist {
        guilds_exist: Vec<u8>,
    },
    /// BaseGuildUserInfo event is described in AnimusMessage
    BaseGuildUserInfo {
        owner_id: String,
        name: String,
        icon: Option<String>,
        /// List of all roles in the server
        roles: Vec<Role>,
        /// List of roles the user has
        user_roles: Vec<RoleId>,
        /// List of roles the bot has
        bot_roles: Vec<RoleId>,
        /// List of all channels in the server
        channels: Vec<GuildChannelWithPermissions>,
    },
    /// Returns the response of a command permission check
    CheckCommandPermission {
        perm_res: PermissionResult,
        is_ok: bool,
    },
    SettingsOperation {
        res: CanonicalSettingsResult,
    },
}

impl AnimusResponse for BotAnimusResponse {
    fn target(&self) -> AnimusTarget {
        AnimusTarget::Bot
    }
}
impl SerializableAnimusResponse for BotAnimusResponse {}

bitflags::bitflags! {
    pub struct AmCheckCommandOptionsFlags: u8 {
        /// Whether or not to ignore the cache
        const IGNORE_CACHE = 1 << 0;
        /// Whether or not to cache the result at all
        const CACHE_RESULT = 1 << 1;
        /// Whether or not to ignore the fact that the module is disabled in the guild
        const IGNORE_MODULE_DISABLED = 1 << 2;
        /// Whether or not to ignore the fact that the command is disabled in the guild
        const IGNORE_COMMAND_DISABLED = 1 << 3;
        /// Skip custom resolved kittycat permission fit 'checks' (AKA does the user have the actual permissions ofthe custom resolved permissions)
        const SKIP_CUSTOM_RESOLVED_FIT_CHECKS = 1 << 4;
    }
}

/// Extra options for checking a command
///
/// This is seperate from the actual internal stuff to both avoid exposing
/// internals and to optimize data flow
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct AmCheckCommandOptions {
    /// Flags of type AmCheckCommandOptionsFlags
    #[serde(default)]
    pub flags: u8,

    /// What custom resolved permissions to use for the user.
    ///
    /// API needs this for limiting the permissions of a user, allows setting custom resolved perms
    #[serde(default)]
    pub custom_resolved_kittycat_perms: Option<Vec<String>>,

    /// Custom permission checks to use
    #[serde(default)]
    pub custom_command_configuration:
        Option<Box<splashcore_rs::types::silverpelt::GuildCommandConfiguration>>,

    /// Custom permission checks to use
    #[serde(default)]
    pub custom_module_configuration:
        Option<Box<splashcore_rs::types::silverpelt::GuildModuleConfiguration>>,
}

#[derive(Serialize, Deserialize, Clone)]
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
    /// Given a guild id, a user id and a command name, check if the user has permission to run the command
    CheckCommandPermission {
        guild_id: GuildId,
        user_id: UserId,
        command: String,
        opts: AmCheckCommandOptions,
    },
    /// Toggles a per-module cache toggle
    ExecutePerModuleFunction {
        module: String,
        toggle: String,
        options: indexmap::IndexMap<String, serde_json::Value>,
    },
    /// Executes an operation on a setting
    SettingsOperation {
        fields: indexmap::IndexMap<String, serde_json::Value>,
        op: module_settings::canonical_types::CanonicalOperationType,
        module: String,
        setting: String,
        guild_id: GuildId,
        user_id: UserId,
    },
}

impl AnimusMessage for BotAnimusMessage {
    fn target(&self) -> AnimusTarget {
        AnimusTarget::Bot
    }
}
impl SerializableAnimusMessage for BotAnimusMessage {}

impl BotAnimusMessage {
    pub async fn response(
        self,
        state: Arc<super::client::ClientData>,
    ) -> Result<BotAnimusResponse, AnimusErrorResponse> {
        let cache_http = &state.cache_http;
        let pool = &state.pool;
        let reqwest = &state.reqwest;

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
                    let has_guild = proxy_support::has_guild(cache_http, reqwest, guild).await?;

                    guilds_exist.push({
                        if has_guild {
                            1
                        } else {
                            0
                        }
                    });
                }

                Ok(BotAnimusResponse::GuildsExist { guilds_exist })
            }
            Self::BaseGuildUserInfo { guild_id, user_id } => {
                let bot_user_id = cache_http.cache.current_user().id;
                let guild = proxy_support::guild(cache_http, reqwest, guild_id)
                    .await
                    .map_err(|e| format!("Failed to get guild: {:#?}", e))?;

                // Next fetch the member and bot_user
                let member: serenity::model::prelude::Member =
                    match proxy_support::member_in_guild(cache_http, reqwest, guild_id, user_id)
                        .await
                    {
                        Ok(Some(member)) => member,
                        Ok(None) => {
                            return Err("Member not found".into());
                        }
                        Err(e) => return Err(format!("Failed to get member: {:#?}", e).into()),
                    };

                let bot_user: serenity::model::prelude::Member =
                    match proxy_support::member_in_guild(cache_http, reqwest, guild_id, bot_user_id)
                        .await
                    {
                        Ok(Some(member)) => member,
                        Ok(None) => {
                            return Err("Bot not found".into());
                        }
                        Err(e) => return Err(format!("Failed to get member: {:#?}", e).into()),
                    };

                // Fetch the channels
                let channels = proxy_support::guild_channels(cache_http, reqwest, guild_id).await?;

                let mut channels_with_permissions = Vec::with_capacity(channels.len());

                for channel in channels.iter() {
                    channels_with_permissions.push(GuildChannelWithPermissions {
                        user: guild.user_permissions_in(channel, &member),
                        bot: guild.user_permissions_in(channel, &bot_user),
                        channel: channel.clone(),
                    });
                }

                Ok(BotAnimusResponse::BaseGuildUserInfo {
                    name: guild.name.to_string(),
                    icon: guild.icon_url(),
                    owner_id: guild.owner_id.to_string(),
                    roles: guild.roles.into_iter().collect(),
                    user_roles: member.roles.to_vec(),
                    bot_roles: bot_user.roles.to_vec(),
                    channels: channels_with_permissions,
                })
            }
            Self::CheckCommandPermission {
                guild_id,
                user_id,
                command,
                opts,
            } => {
                // Check COMMAND_ID_MODULE_MAP
                let base_command = command.split_whitespace().next().unwrap();

                let flags = AmCheckCommandOptionsFlags::from_bits_truncate(opts.flags);

                let perm_res = modules::silverpelt::cmd::check_command(
                    base_command,
                    &command,
                    guild_id,
                    user_id,
                    pool,
                    cache_http,
                    &None,
                    modules::silverpelt::cmd::CheckCommandOptions {
                        ignore_cache: flags.contains(AmCheckCommandOptionsFlags::IGNORE_CACHE),
                        cache_result: flags.contains(AmCheckCommandOptionsFlags::CACHE_RESULT),
                        ignore_module_disabled: flags
                            .contains(AmCheckCommandOptionsFlags::IGNORE_MODULE_DISABLED),
                        ignore_command_disabled: flags
                            .contains(AmCheckCommandOptionsFlags::IGNORE_COMMAND_DISABLED),
                        custom_resolved_kittycat_perms: opts.custom_resolved_kittycat_perms.map(
                            |crkp| {
                                crkp.iter()
                                    .map(|x| kittycat::perms::Permission::from_string(x))
                                    .collect::<Vec<kittycat::perms::Permission>>()
                            },
                        ),
                        custom_command_configuration: opts.custom_command_configuration.map(|x| *x),
                        custom_module_configuration: opts.custom_module_configuration.map(|x| *x),
                        skip_custom_resolved_fit_checks: flags
                            .contains(AmCheckCommandOptionsFlags::SKIP_CUSTOM_RESOLVED_FIT_CHECKS),
                    },
                )
                .await;

                let is_ok = perm_res.is_ok();

                Ok(BotAnimusResponse::CheckCommandPermission { perm_res, is_ok })
            }
            Self::ExecutePerModuleFunction {
                module,
                toggle,
                options,
            } => {
                let Some(toggle) =
                    crate::PERMODULE_FUNCTIONS.get(&(module.clone(), toggle.clone()))
                else {
                    return Err("Toggle not found".into());
                };

                let mut n_options = indexmap::IndexMap::new();

                for (k, v) in options {
                    n_options.insert(k, Value::from_json(&v));
                }

                (toggle)(cache_http, &n_options).await?;

                Ok(BotAnimusResponse::Ok {
                    message: "".to_string(),
                })
            }
            Self::SettingsOperation {
                fields,
                op,
                module,
                setting,
                guild_id,
                user_id,
            } => {
                let op: OperationType = op.into();

                // Find the setting
                let Some(module) = SILVERPELT_CACHE.module_cache.get(&module) else {
                    return Ok(BotAnimusResponse::SettingsOperation {
                        res: CanonicalSettingsResult::Err {
                            error: CanonicalSettingsError::Generic {
                                message: "Module not found".to_string(),
                                src: "SettingsOperation".to_string(),
                                typ: "badRequest".to_string(),
                            },
                        },
                    });
                };

                let Some(opt) = module.config_options.iter().find(|x| x.id == setting) else {
                    return Ok(BotAnimusResponse::SettingsOperation {
                        res: CanonicalSettingsResult::Err {
                            error: CanonicalSettingsError::Generic {
                                message: "Setting not found".to_string(),
                                src: "SettingsOperation".to_string(),
                                typ: "badRequest".to_string(),
                            },
                        },
                    });
                };

                let mut p_fields = indexmap::IndexMap::new();

                // As the order of fields may not be guaranteed, we need to add the fields in the order of the columns
                //
                // We then add the rest of the fields not in columns as well
                for column in opt.columns.iter() {
                    if let Some(value) = fields.get(column.id) {
                        p_fields.insert(column.id.to_string(), Value::from_json(value));
                    }
                }

                // Add the rest of the fields
                for (key, value) in fields {
                    if p_fields.contains_key(&key) {
                        continue;
                    }

                    p_fields.insert(key, Value::from_json(&value));
                }

                let Some(operation_specific) = opt.operations.get(&op) else {
                    return Ok(BotAnimusResponse::SettingsOperation {
                        res: CanonicalSettingsResult::Err {
                            error: CanonicalSettingsError::OperationNotSupported {
                                operation: op.into(),
                            },
                        },
                    });
                };

                // Check COMMAND_ID_MODULE_MAP
                let base_command = operation_specific
                    .corresponding_command
                    .split_whitespace()
                    .next()
                    .unwrap();

                let perm_res = modules::silverpelt::cmd::check_command(
                    base_command,
                    operation_specific.corresponding_command,
                    guild_id,
                    user_id,
                    pool,
                    cache_http,
                    &None,
                    modules::silverpelt::cmd::CheckCommandOptions {
                        ignore_module_disabled: true,
                        ..Default::default()
                    },
                )
                .await;

                if !perm_res.is_ok() {
                    return Ok(BotAnimusResponse::SettingsOperation {
                        res: CanonicalSettingsResult::PermissionError { res: perm_res },
                    });
                }

                match op {
                    OperationType::View => {
                        match module_settings::cfg::settings_view(
                            opt,
                            &state.cache_http,
                            &state.reqwest,
                            pool,
                            guild_id,
                            user_id,
                            &crate::PermoduleFunctionExecutor {},
                        )
                        .await
                        {
                            Ok(res) => Ok(BotAnimusResponse::SettingsOperation {
                                res: CanonicalSettingsResult::Ok {
                                    fields: res.into_iter().map(|x| x.into()).collect(),
                                },
                            }),
                            Err(e) => Ok(BotAnimusResponse::SettingsOperation {
                                res: CanonicalSettingsResult::Err { error: e.into() },
                            }),
                        }
                    }
                    OperationType::Create => {
                        match module_settings::cfg::settings_create(
                            opt,
                            &state.cache_http,
                            &state.reqwest,
                            pool,
                            guild_id,
                            user_id,
                            p_fields,
                            &crate::PermoduleFunctionExecutor {},
                        )
                        .await
                        {
                            Ok(res) => Ok(BotAnimusResponse::SettingsOperation {
                                res: CanonicalSettingsResult::Ok {
                                    fields: vec![res.into()],
                                },
                            }),
                            Err(e) => Ok(BotAnimusResponse::SettingsOperation {
                                res: CanonicalSettingsResult::Err { error: e.into() },
                            }),
                        }
                    }
                    OperationType::Update => {
                        match module_settings::cfg::settings_update(
                            opt,
                            &state.cache_http,
                            &state.reqwest,
                            pool,
                            guild_id,
                            user_id,
                            p_fields,
                            &crate::PermoduleFunctionExecutor {},
                        )
                        .await
                        {
                            Ok(res) => Ok(BotAnimusResponse::SettingsOperation {
                                res: CanonicalSettingsResult::Ok {
                                    fields: vec![res.into()],
                                },
                            }),
                            Err(e) => Ok(BotAnimusResponse::SettingsOperation {
                                res: CanonicalSettingsResult::Err { error: e.into() },
                            }),
                        }
                    }
                    OperationType::Delete => {
                        let Some(pkey) = p_fields.get(opt.primary_key) else {
                            return Ok(BotAnimusResponse::SettingsOperation {
                                res: CanonicalSettingsResult::Err {
                                    error: CanonicalSettingsError::MissingOrInvalidField {
                                        field: opt.primary_key.to_string(),
                                        src: "SettingsOperation".to_string(),
                                    },
                                },
                            });
                        };

                        match module_settings::cfg::settings_delete(
                            opt,
                            &state.cache_http,
                            &state.reqwest,
                            pool,
                            guild_id,
                            user_id,
                            pkey.clone(),
                            &crate::PermoduleFunctionExecutor {},
                        )
                        .await
                        {
                            Ok(res) => Ok(BotAnimusResponse::SettingsOperation {
                                res: CanonicalSettingsResult::Ok {
                                    fields: vec![res.into()],
                                },
                            }),
                            Err(e) => Ok(BotAnimusResponse::SettingsOperation {
                                res: CanonicalSettingsResult::Err { error: e.into() },
                            }),
                        }
                    }
                }
            }
        }
    }
}
