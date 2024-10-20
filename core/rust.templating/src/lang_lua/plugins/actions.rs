use crate::lang_lua::state;
use governor::clock::Clock;
use mlua::prelude::*;
use std::sync::Arc;

/// A ban action
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BanAction {
    user_id: serenity::all::UserId,
    reason: String,
    delete_message_days: Option<u8>,
}

/// A kick action
#[derive(serde::Serialize, serde::Deserialize)]
pub struct KickAction {
    user_id: serenity::all::UserId,
    reason: String,
}

/// A timeout action
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TimeoutAction {
    user_id: serenity::all::UserId,
    reason: String,
    duration_seconds: u64,
}

/// A kick action
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SendMessageChannelAction {
    channel_id: serenity::all::ChannelId, // Channel *must* be in the same guild
    message: crate::core::messages::Message,
}

/// An action executor is used to execute actions such as kick/ban/timeout from Lua
/// templates
pub struct ActionExecutor {
    template_data: Arc<state::TemplateData>,
    guild_id: serenity::all::GuildId,
    cache_http: botox::cache::CacheHttpImpl,
    reqwest_client: reqwest::Client,
    ratelimits: Arc<state::LuaActionsRatelimit>,
}

impl ActionExecutor {
    pub fn check_action(&self, action: String) -> Result<(), crate::Error> {
        if !self.template_data.pragma.actions.contains(&action) {
            return Err("Action not allowed in this template context".into());
        }

        // Check global ratelimits
        for global_lim in self.ratelimits.global.iter() {
            match global_lim.check_key(&()) {
                Ok(()) => continue,
                Err(wait) => {
                    return Err(format!(
                        "Global ratelimit hit for action '{}', wait time: {:?}",
                        action,
                        wait.wait_time_from(self.ratelimits.clock.now())
                    )
                    .into());
                }
            };
        }

        // Check per bucket ratelimits
        if let Some(per_bucket) = self.ratelimits.per_bucket.get(&action) {
            for lim in per_bucket.iter() {
                match lim.check_key(&()) {
                    Ok(()) => continue,
                    Err(wait) => {
                        return Err(format!(
                            "Per bucket ratelimit hit for action '{}', wait time: {:?}",
                            action,
                            wait.wait_time_from(self.ratelimits.clock.now())
                        )
                        .into());
                    }
                };
            }
        }

        Ok(())
    }
}

impl LuaUserData for ActionExecutor {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("ban", |lua, this, data: LuaValue| async move {
            let data = lua.from_value::<BanAction>(data)?;

            this.check_action("ban".to_string())
                .map_err(|e| LuaError::external(e))?;

            let delete_message_days = {
                if let Some(days) = data.delete_message_days {
                    if days > 7 {
                        return Err(LuaError::external(
                            "Delete message days must be between 0 and 7",
                        ));
                    }

                    days
                } else {
                    0
                }
            };

            if data.reason.len() > 128 || data.reason.is_empty() {
                return Err(LuaError::external(
                    "Reason must be less than 128 characters and not empty",
                ));
            }

            this.cache_http
                .http
                .ban_user(
                    this.guild_id,
                    data.user_id,
                    delete_message_days,
                    Some(data.reason.as_str()),
                )
                .await
                .map_err(|e| LuaError::external(e))?;

            Ok(())
        });

        methods.add_async_method("kick", |lua, this, data: LuaValue| async move {
            let data = lua.from_value::<KickAction>(data)?;

            this.check_action("kick".to_string())
                .map_err(|e| LuaError::external(e))?;

            if data.reason.len() > 128 || data.reason.is_empty() {
                return Err(LuaError::external(
                    "Reason must be less than 128 characters and not empty",
                ));
            }

            this.cache_http
                .http
                .kick_member(this.guild_id, data.user_id, Some(data.reason.as_str()))
                .await
                .map_err(|e| LuaError::external(e))?;

            Ok(())
        });

        methods.add_async_method("timeout", |lua, this, data: LuaValue| async move {
            let data = lua.from_value::<TimeoutAction>(data)?;

            this.check_action("timeout".to_string())
                .map_err(|e| LuaError::external(e))?;

            if data.reason.len() > 128 || data.reason.is_empty() {
                return Err(LuaError::external(
                    "Reason must be less than 128 characters and not empty",
                ));
            }

            if data.duration_seconds > 60 * 60 * 24 * 28 {
                return Err(LuaError::external(
                    "Timeout duration must be less than 28 days",
                ));
            }

            let communication_disabled_until =
                chrono::Utc::now() + std::time::Duration::from_secs(data.duration_seconds);

            this.guild_id
                .edit_member(
                    &this.cache_http.http,
                    data.user_id,
                    serenity::all::EditMember::new()
                        .audit_log_reason(data.reason.as_str())
                        .disable_communication_until(communication_disabled_until.into()),
                )
                .await
                .map_err(|e| LuaError::external(e))?;

            Ok(())
        });

        methods.add_async_method(
            "sendmessage_channel",
            |lua, this, data: LuaValue| async move {
                let data = lua.from_value::<SendMessageChannelAction>(data)?;

                this.check_action("sendmessage_channel".to_string())
                    .map_err(|e| LuaError::external(e))?;

                let msg = crate::core::messages::to_discord_reply(data.message)
                    .map_err(|e| LuaError::external(e))?;

                // Perform required checks
                let channel = sandwich_driver::channel(
                    &this.cache_http,
                    &this.reqwest_client,
                    Some(this.guild_id),
                    data.channel_id,
                )
                .await
                .map_err(|e| LuaError::external(e))?;

                let Some(channel) = channel else {
                    return Err(LuaError::external("Channel not found"));
                };

                let Some(guild_channel) = channel.guild() else {
                    return Err(LuaError::external("Channel not in guild"));
                };

                if guild_channel.guild_id != this.guild_id {
                    return Err(LuaError::external("Channel not in guild"));
                }

                let bot_user_id = this.cache_http.cache.current_user().id;

                let bot_user = sandwich_driver::member_in_guild(
                    &this.cache_http,
                    &this.reqwest_client,
                    this.guild_id,
                    bot_user_id,
                )
                .await
                .map_err(|e| LuaError::external(e))?;

                let Some(bot_user) = bot_user else {
                    return Err(LuaError::external("Bot user not found"));
                };

                let guild =
                    sandwich_driver::guild(&this.cache_http, &this.reqwest_client, this.guild_id)
                        .await
                        .map_err(|e| LuaError::external(e))?;

                // Check if the bot has permissions to send messages in the given channel
                if !guild
                    .user_permissions_in(&guild_channel, &bot_user)
                    .send_messages()
                {
                    return Err(LuaError::external(
                        "Bot does not have permission to send messages in the given channel",
                    ));
                }

                let mut cm = serenity::all::CreateMessage::new();

                if let Some(content) = msg.content {
                    cm = cm.content(content);
                }

                cm = cm.embeds(msg.embeds);

                guild_channel
                    .send_message(&this.cache_http.http, cm)
                    .await
                    .map_err(|e| LuaError::external(e))?;

                Ok(())
            },
        );
    }
}

pub fn init_plugin(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    module.set(
        "new",
        lua.create_function(|lua, (token,): (String,)| {
            let Some(data) = lua.app_data_ref::<state::LuaUserData>() else {
                return Err(LuaError::external("No app data found"));
            };

            let template_data = data
                .per_template
                .get(&token)
                .ok_or_else(|| LuaError::external("Template not found"))?;

            let executor = ActionExecutor {
                template_data: template_data.clone(),
                guild_id: data.guild_id.clone(),
                cache_http: data.cache_http.clone(),
                reqwest_client: data.reqwest_client.clone(),
                ratelimits: data.ratelimits.clone(),
            };

            Ok(executor)
        })?,
    )?;

    module.set_readonly(true); // Block any attempt to modify this table

    Ok(module)
}
