use super::dehoist::dehoist_user;
use super::types::{
    DehoistOptions, FakeBotDetectionOptions, GuildProtectionOptions, TriggeredFlags, MAX_MENTIONS,
};
use crate::{
    silverpelt::{module_config::is_module_enabled, EventHandlerContext},
    Error,
};
use proxy_support::member_in_guild;
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

            if !config
                .fake_bot_detection
                .contains(FakeBotDetectionOptions::DISABLED)
                && new_member.user.bot()
            {
                let mut found = false;

                if config
                    .fake_bot_detection
                    .contains(FakeBotDetectionOptions::BLOCK_ALL_BOTS)
                {
                    // Doesn't matter if its official or not, the server wants to block all bots
                    found = true;
                } else if config
                    .fake_bot_detection
                    .contains(FakeBotDetectionOptions::BLOCK_ALL_UNKNOWN_BOTS)
                {
                    // Check if the bot is an official bot or not
                    let is_an_official_bot = super::cache::FAKE_BOTS_CACHE
                        .iter()
                        .any(|fb| fb.value().official_bot_ids.contains(&new_member.user.id));

                    if !is_an_official_bot {
                        found = true;
                    }
                } else {
                    // Normalize the bots name
                    let normalized_name = if config
                        .fake_bot_detection
                        .contains(FakeBotDetectionOptions::NORMALIZE_NAMES)
                    {
                        let mut normalized_name = splashcore_text::normalize::fix_text(
                            &new_member.user.name.to_lowercase(),
                            None,
                        );

                        // Handle prefixes people add to the bot name for scamming by removing them
                        // TODO: Make a database of these
                        for prefixes in ["premium", "vip", "prime", "pro", "official", "bot"].iter()
                        {
                            if normalized_name.starts_with(prefixes) {
                                normalized_name = normalized_name[prefixes.len()..].to_string();
                            }
                        }

                        // Handle suffixes people add to the bot name for scamming by removing them
                        for suffixes in ["premium", "vip", "prime", "pro", "official", "bot"].iter()
                        {
                            if normalized_name.starts_with(suffixes) {
                                normalized_name = normalized_name[suffixes.len()..].to_string();
                            }
                        }

                        // Trim the name
                        normalized_name = normalized_name.trim().to_string();

                        normalized_name
                    } else {
                        new_member.user.name.to_lowercase()
                    };

                    for fb in super::cache::FAKE_BOTS_CACHE.iter() {
                        let val = fb.value();

                        if val.official_bot_ids.contains(&new_member.user.id) {
                            found = false; // Official bot
                            break;
                        }

                        if config
                            .fake_bot_detection
                            .contains(FakeBotDetectionOptions::EXACT_NAME_CHECK)
                            && normalized_name.contains(&val.name)
                        {
                            found = true;
                            break;
                        }

                        if config
                            .fake_bot_detection
                            .contains(FakeBotDetectionOptions::SIMILAR_NAME_CHECK)
                        {
                            let (diff, _) =
                                splashcore_text::diff::diff(&normalized_name, &val.name, "");

                            if diff <= 2 {
                                found = true;
                                break;
                            }
                        }
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

                // Send audit logs if Audit Logs module is enabled
                if crate::silverpelt::module_config::is_module_enabled(
                    &data.pool,
                    ectx.guild_id,
                    "auditlogs",
                )
                .await?
                {
                    let imap = indexmap::indexmap! {
                        "member".to_string() => gwevent::field::CategorizedField { category: "summary".to_string(), field: new_member.user.clone().into() },
                        "triggered_flags".to_string() => gwevent::field::CategorizedField { category: "summary".to_string(), field: triggered_flags.iter_names().map(|(flag, _)| flag.to_string()).collect::<Vec<String>>().join(", ").into() },
                    };

                    crate::modules::auditlogs::events::dispatch_audit_log(
                        ctx,
                        "AR/MemberJoinInspectionFailed",
                        "(Anti-Raid) Member Join Inspection Failed",
                        imap,
                        ectx.guild_id,
                    )
                    .await?;
                }
            }

            // Lastly, check for hoisting attempts
            if !config.hoist_detection.contains(DehoistOptions::DISABLED) {
                let display_name = new_member.display_name().to_string();
                let new = dehoist_user(&display_name, config.hoist_detection);

                if new != display_name {
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
        FullEvent::GuildUpdate {
            old_data_if_available,
            new_data,
        } => {
            let name_changed = {
                if let Some(old_data) = old_data_if_available {
                    old_data.name != new_data.name
                } else {
                    true // Be safe here
                }
            };

            let icon_changed = {
                if let Some(old_data) = old_data_if_available {
                    old_data.icon != new_data.icon
                } else {
                    true // Be safe here
                }
            };

            let config = super::cache::get_config(&ectx.data.pool, ectx.guild_id).await?;

            if !config
                .guild_protection
                .contains(GuildProtectionOptions::DISABLED)
            {
                let bot_userid = ectx.serenity_context.cache.current_user().id;
                let cache_http = botox::cache::CacheHttpImpl::from_ctx(&ectx.serenity_context);
                let Some(bot) =
                    member_in_guild(&cache_http, &ectx.data.reqwest, ectx.guild_id, bot_userid)
                        .await?
                else {
                    return Err("Bot member not found".into());
                };

                let bp = bot.permissions(&ectx.serenity_context.cache)?;

                if !bp.manage_guild() {
                    return Err(
                        format!("Cannot manage guild on this guild: {}", ectx.guild_id).into(),
                    );
                }

                let Some(row) = sqlx::query!(
                    "SELECT name, icon FROM inspector__guilds WHERE guild_id = $1",
                    ectx.guild_id.to_string(),
                )
                .fetch_optional(&ectx.data.pool)
                .await?
                else {
                    return Ok(()); // No row to revert to
                };

                super::guildprotect::Snapshot {
                    guild_id: ectx.guild_id,
                    name: row.name.clone(),
                    icon: row.icon.clone(),
                }
                .revert(ctx, &ectx.data, name_changed, icon_changed)
                .await?;
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
