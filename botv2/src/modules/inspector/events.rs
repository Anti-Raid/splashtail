use super::dehoist::dehoist_user;
use super::types::{DehoistOptions, TriggeredFlags, MAX_MENTIONS};
use crate::{
    silverpelt::{
        module_config::is_module_enabled, proxysupport::member_in_guild, EventHandlerContext,
    },
    Error,
};
use serenity::all::FullEvent;

pub async fn event_listener(ectx: &EventHandlerContext) -> Result<(), Error> {
    let ctx = &ectx.serenity_context;
    let event = &ectx.full_event;

    match event {
        FullEvent::Message { new_message } => {
            let data = &ectx.data;
            let config = super::cache::get_config(&data.pool, ectx.guild_id).await?;

            let mut triggered_flags = TriggeredFlags::NONE;
            let mut triggered_stings = 0;

            if let Some(ai_stings) = config.anti_invite {
                let trimmed_msg = new_message
                    .content
                    .trim()
                    .replace("dot", ".")
                    .replace("slash", "/")
                    .replace(['(', ')'], "");

                if trimmed_msg.contains("discord.gg")
                    || trimmed_msg.contains("discordapp.com/invite")
                    || trimmed_msg.contains("discord.com/invite")
                {
                    triggered_flags |= TriggeredFlags::ANTI_INVITE;
                    triggered_stings += ai_stings;
                }
            }

            if let Some(ae_stings) = config.anti_everyone {
                if new_message.content.contains("@everyone")
                    || new_message.mention_everyone()
                    || new_message.mentions.len() > MAX_MENTIONS
                {
                    triggered_flags |= TriggeredFlags::ANTI_EVERYONE;
                    triggered_stings += ae_stings;
                }
            }

            if triggered_flags != TriggeredFlags::NONE {
                // For now, don't do anything, punishment support is coming soon
                new_message
                    .delete(
                        &ctx.http,
                        Some(&format!("Message triggered flags: {:?}", {
                            let mut tf = vec![];

                            for (name, _) in triggered_flags.iter_names() {
                                tf.push(name);
                            }

                            tf.join(", ")
                        })),
                    )
                    .await?;

                // Apply stings
                if triggered_stings > 0
                    && is_module_enabled(&data.pool, ectx.guild_id, "punishments").await?
                {
                    sqlx::query!(
                        "INSERT INTO inspector__punishments (user_id, guild_id, stings, triggered_flags) VALUES ($1, $2, $3, $4)",
                        new_message.author.id.to_string(),
                        ectx.guild_id.to_string(),
                        triggered_stings as i32,
                        i64::from(triggered_flags.bits())
                    )
                    .execute(&data.pool)
                    .await?;
                }
            }

            Ok(())
        }
        FullEvent::GuildMemberAddition { new_member } => {
            let data = &ectx.data;
            let config = super::cache::get_config(&data.pool, ectx.guild_id).await?;

            let mut triggered_flags = TriggeredFlags::NONE;

            if let Some(minimum_account_age) = config.minimum_account_age {
                let account_age = new_member.user.created_at();
                let now = chrono::Utc::now();
                if let Some(duration) = chrono::Duration::try_seconds(minimum_account_age) {
                    let diff = now - *account_age;

                    if diff < duration {
                        triggered_flags |= TriggeredFlags::MINIMUM_ACCOUNT_AGE;
                    }
                }
            }

            if let Some(maximum_account_age) = config.maximum_account_age {
                let account_age = new_member.user.created_at();
                let now = chrono::Utc::now();

                if let Some(duration) = chrono::Duration::try_seconds(maximum_account_age) {
                    let diff = now - *account_age;

                    if diff > duration {
                        triggered_flags |= TriggeredFlags::MAXIMUM_ACCOUNT_AGE;
                    }
                }
            }

            if config.fake_bot_detection && new_member.user.bot() {
                // Normalize the bots name
                let normalized_name = plsfix::fix_text(&new_member.user.name.to_lowercase(), None);

                let mut found = false;
                for fb in super::cache::FAKE_BOTS_CACHE.iter() {
                    let val = fb.value();

                    if val.official_bot_ids.contains(&new_member.user.id) {
                        continue;
                    }

                    if normalized_name.contains(&val.name) {
                        found = true;
                        break;
                    }

                    let (diff, _) = text_diff::diff(&normalized_name, &val.name, "");

                    if diff < 2 {
                        found = true;
                        break;
                    }
                }

                if found {
                    triggered_flags |= TriggeredFlags::FAKE_BOT_DETECTION;
                }
            }

            if triggered_flags != TriggeredFlags::NONE {
                let bot_userid = ectx.serenity_context.cache.current_user().id;
                let cache_http = botox::cache::CacheHttpImpl::from_ctx(&ectx.serenity_context);
                let Some(bot) =
                    member_in_guild(&cache_http, &data.reqwest, ectx.guild_id, bot_userid).await?
                else {
                    return Err("Bot member not found".into());
                };

                // TODO: Check for hierarchy here too

                let bp = bot.permissions(&ectx.serenity_context.cache)?;

                if !bp.kick_members() {
                    return Err(
                        format!("Cannot kick members on this guild: {}", ectx.guild_id).into(),
                    );
                }

                new_member
                    .kick(
                        &ctx.http,
                        Some(&format!("User triggered flags: {:?}", {
                            let mut tf = vec![];

                            for (name, _) in triggered_flags.iter_names() {
                                tf.push(name);
                            }

                            tf.join(", ")
                        })),
                    )
                    .await?;
            }

            // Lastly, check for hoisting attempts
            if !config.hoist_detection.contains(DehoistOptions::DISABLED) {
                let display_name = new_member.display_name().to_string();
                let new = dehoist_user(&display_name, config.hoist_detection);

                if display_name != new {
                    let bot_userid = ectx.serenity_context.cache.current_user().id;
                    let cache_http = botox::cache::CacheHttpImpl::from_ctx(&ectx.serenity_context);
                    let Some(bot) =
                        member_in_guild(&cache_http, &data.reqwest, ectx.guild_id, bot_userid)
                            .await?
                    else {
                        return Err("Bot member not found".into());
                    };

                    // TODO: Check for hierarchy here too

                    let bp = bot.permissions(&ectx.serenity_context.cache)?;

                    if !bp.manage_nicknames() {
                        return Err(format!(
                            "Cannot manage nicknames on this guild: {}",
                            ectx.guild_id
                        )
                        .into());
                    }

                    let mut new_member = new_member.clone();
                    new_member
                        .edit(
                            &ctx.http,
                            serenity::all::EditMember::new()
                                .nickname(new)
                                .audit_log_reason(
                                    "User attempted to hoist themselves on the member list",
                                ),
                        )
                        .await?;
                }
            }

            Ok(())
        }
        FullEvent::GuildMemberUpdate {
            old_if_available: _,
            new: _,
            event,
        } => {
            let data = &ectx.data;
            let config = super::cache::get_config(&data.pool, ectx.guild_id).await?;

            // Hoist detection
            if !config.hoist_detection.contains(DehoistOptions::DISABLED) {
                let display_name = event
                    .nick
                    .as_ref()
                    .or(event.user.global_name.as_ref())
                    .unwrap_or(&event.user.name)
                    .to_string();

                let new = dehoist_user(&display_name, config.hoist_detection);

                if display_name != new {
                    let bot_userid = ectx.serenity_context.cache.current_user().id;
                    let cache_http = botox::cache::CacheHttpImpl::from_ctx(&ectx.serenity_context);
                    let Some(bot) =
                        member_in_guild(&cache_http, &data.reqwest, ectx.guild_id, bot_userid)
                            .await?
                    else {
                        return Err("Bot member not found".into());
                    };

                    let bp = bot.permissions(&ectx.serenity_context.cache)?;

                    // TODO: Check for hierarchy here too

                    if !bp.manage_nicknames() {
                        return Err(format!(
                            "Cannot manage nicknames on this guild: {}",
                            ectx.guild_id
                        )
                        .into());
                    }

                    event
                        .guild_id
                        .edit_member(
                            &ctx.http,
                            event.user.id,
                            serenity::all::EditMember::new()
                                .nickname(new)
                                .audit_log_reason(
                                    "User attempted to hoist themselves on the member list",
                                ),
                        )
                        .await?;
                }
            }

            Ok(())
        }
        _ => Ok(()),
    }
}
