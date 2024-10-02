use super::dehoist::dehoist_user;
use super::types::{
    AutoResponseMemberJoinOptions, DehoistOptions, FakeBotDetectionOptions, GuildProtectionOptions,
    TriggeredFlags, MAX_MENTIONS,
};
use proxy_support::{guild, member_in_guild};
use serenity::all::FullEvent;
use silverpelt::Error;
use silverpelt::{module_config::is_module_enabled, EventHandlerContext};

pub async fn event_listener(ectx: &EventHandlerContext) -> Result<(), Error> {
    let ctx = &ectx.serenity_context;
    let event = &ectx.full_event;

    match event {
        FullEvent::Message { new_message } => {
            let data = &ectx.data;
            let config = super::cache::get_specific_configs(&data.pool, ectx.guild_id).await?;

            let anti_invite = super::cache::InspectorSpecificOptions::get(
                &config,
                |c| c.anti_invite,
                new_message.author.id,
                Some(new_message.channel_id),
            );
            let anti_everyone = super::cache::InspectorSpecificOptions::get(
                &config,
                |c| c.anti_everyone,
                new_message.author.id,
                Some(new_message.channel_id),
            );

            let mut triggered_flags = TriggeredFlags::NONE;
            let mut triggered_stings = 0;

            if let Some(ai_stings) = anti_invite {
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

            if let Some(ae_stings) = anti_everyone {
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
                    && is_module_enabled(
                        &data.silverpelt_cache,
                        &data.pool,
                        ectx.guild_id,
                        "punishments",
                    )
                    .await?
                {
                    // Create a new sting
                    let sting_retention = super::cache::InspectorSpecificOptions::get(
                        &config,
                        |c| Some(c.sting_retention),
                        new_message.author.id,
                        Some(new_message.channel_id),
                    )
                    .unwrap_or(300);

                    silverpelt::stings::StingCreate {
                        module: "inspector".to_string(),
                        src: None,
                        stings: triggered_stings,
                        reason: Some(format!(
                            "Message triggered flags: {:?}",
                            triggered_flags.iter_names().map(|(flag, _)| flag.to_string()).collect::<Vec<String>>().join(", ")
                        )),
                        void_reason: None,
                        guild_id: ectx.guild_id,
                        creator: silverpelt::stings::StingTarget::System,
                        target: silverpelt::stings::StingTarget::User(new_message.author.id),
                        state: silverpelt::stings::StingState::Active,
                        duration: Some(std::time::Duration::from_secs(sting_retention as u64)),
                        sting_data: Some(serde_json::json!({
                            "triggered_flags": triggered_flags.bits(),
                        })),
                        handle_log: None,
                        punishment: None,
                    }
                    .create(&data.pool)
                    .await?;

                    // Trigger punishment
                    bot_modules_punishments::core::trigger_punishment(
                        &ctx,
                        ectx.guild_id,
                    )
                    .await?;
                }
            }

            Ok(())
        }
        FullEvent::GuildMemberAddition { new_member } => {
            let data = &ectx.data;
            let config = super::cache::get_global_config(&data.pool, ectx.guild_id).await?;

            // First check for an auto response
            if !config
                .auto_response_memberjoin
                .contains(AutoResponseMemberJoinOptions::DISABLED)
            {
                let bot_userid = ectx.serenity_context.cache.current_user().id;
                let cache_http = botox::cache::CacheHttpImpl::from_ctx(&ectx.serenity_context);
                let Some(bot) =
                    member_in_guild(&cache_http, &data.reqwest, ectx.guild_id, bot_userid).await?
                else {
                    return Err("Bot member not found".into());
                };

                let bp = bot.permissions(&ectx.serenity_context.cache)?;

                // keep going through the list of responses until we have one that works
                for flag in AutoResponseMemberJoinOptions::order() {
                    if config.auto_response_memberjoin.contains(flag) {
                        match flag {
                            AutoResponseMemberJoinOptions::DISABLED => break,
                            AutoResponseMemberJoinOptions::KICK_NEW_MEMBERS => {
                                if !bp.kick_members() {
                                    log::error!(
                                        "Cannot kick members on this guild: {}",
                                        ectx.guild_id
                                    );
                                    continue; // Try the next one
                                }

                                new_member
                                    .kick(&ctx.http, Some("Auto response: Kick new members"))
                                    .await?;

                                // Send audit logs if Audit Logs module is enabled
                                if silverpelt::module_config::is_module_enabled(
                                    &data.silverpelt_cache,
                                    &data.pool,
                                    ectx.guild_id,
                                    "auditlogs",
                                )
                                .await?
                                {
                                    let imap = indexmap::indexmap! {
                                        "member".to_string() => gwevent::field::CategorizedField { category: "summary".to_string(), field: new_member.user.clone().into() },
                                    };

                                    bot_modules_auditlogs::events::dispatch_audit_log(
                                        ctx,
                                        data,
                                        "AR/Inspector_AutoResponseMemberJoin.KickNewMembers",
                                        "(Anti-Raid) Auto Response: Kick New Members",
                                        imap,
                                        ectx.guild_id,
                                    )
                                    .await?;
                                }
                            }
                            AutoResponseMemberJoinOptions::BAN_NEW_MEMBERS => {
                                if !bp.ban_members() {
                                    log::error!(
                                        "Cannot ban members on this guild: {}",
                                        ectx.guild_id
                                    );
                                    continue; // Try the next one
                                }

                                new_member
                                    .ban(&ctx.http, 0, Some("Auto response: Ban new members"))
                                    .await?;

                                // Send audit logs if Audit Logs module is enabled
                                if silverpelt::module_config::is_module_enabled(
                                    &data.silverpelt_cache,
                                    &data.pool,
                                    ectx.guild_id,
                                    "auditlogs",
                                )
                                .await?
                                {
                                    let imap = indexmap::indexmap! {
                                        "member".to_string() => gwevent::field::CategorizedField { category: "summary".to_string(), field: new_member.user.clone().into() },
                                    };

                                    bot_modules_auditlogs::events::dispatch_audit_log(
                                        ctx,
                                        data,
                                        "AR/Inspector_AutoResponseMemberJoin.BanNewMembers",
                                        "(Anti-Raid) Auto Response Ban New Members",
                                        imap,
                                        ectx.guild_id,
                                    )
                                    .await?;
                                }
                            }
                            _ => continue, // Ignore unknown flags
                        }
                    }
                }
            }

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
                if silverpelt::module_config::is_module_enabled(
                    &data.silverpelt_cache,
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

                    bot_modules_auditlogs::events::dispatch_audit_log(
                        ctx,
                        data,
                        "AR/Inspector_MemberJoinInspectionFailed",
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
                                .nickname(new.clone())
                                .audit_log_reason(
                                    "User attempted to hoist themselves on the member list",
                                ),
                        )
                        .await?;

                    // Send audit logs if Audit Logs module is enabled
                    if silverpelt::module_config::is_module_enabled(
                        &data.silverpelt_cache,
                        &data.pool,
                        ectx.guild_id,
                        "auditlogs",
                    )
                    .await?
                    {
                        let imap = indexmap::indexmap! {
                            "member".to_string() => gwevent::field::CategorizedField { category: "summary".to_string(), field: new_member.user.clone().into() },
                            "old_display_name".to_string() => gwevent::field::CategorizedField { category: "summary".to_string(), field: display_name.into() },
                            "new_nickname".to_string() => gwevent::field::CategorizedField { category: "summary".to_string(), field: new.into() },
                        };

                        bot_modules_auditlogs::events::dispatch_audit_log(
                            ctx,
                            data,
                            "AR/Inspector_MemberJoinHoistAttempt",
                            "(Anti-Raid) Member Join Hoist Attempt",
                            imap,
                            ectx.guild_id,
                        )
                        .await?;
                    }
                }
            }

            Ok(())
        }
        FullEvent::GuildUpdate {
            old_data_if_available,
            new_data,
        } => {
            log::info!("Guild update event [inspector]: {}", new_data.id);
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

            let config = super::cache::get_global_config(&ectx.data.pool, ectx.guild_id).await?;

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
                .revert(
                    ctx,
                    &ectx.data,
                    name_changed
                        && config
                            .guild_protection
                            .contains(GuildProtectionOptions::NAME),
                    icon_changed
                        && config
                            .guild_protection
                            .contains(GuildProtectionOptions::ICON),
                )
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
            let config = super::cache::get_global_config(&data.pool, ectx.guild_id).await?;

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

                    let guild = guild(&cache_http, &data.reqwest, ectx.guild_id).await?;

                    let bp = guild.member_permissions(&bot);

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
                                .nickname(new.clone())
                                .audit_log_reason(
                                    "User attempted to hoist themselves on the member list",
                                ),
                        )
                        .await?;

                    // Send audit logs if Audit Logs module is enabled
                    if silverpelt::module_config::is_module_enabled(
                        &data.silverpelt_cache,
                        &data.pool,
                        ectx.guild_id,
                        "auditlogs",
                    )
                    .await?
                    {
                        let imap = indexmap::indexmap! {
                            "member".to_string() => gwevent::field::CategorizedField { category: "summary".to_string(), field: event.user.clone().into() },
                            "old_display_name".to_string() => gwevent::field::CategorizedField { category: "summary".to_string(), field: display_name.into() },
                            "new_nickname".to_string() => gwevent::field::CategorizedField { category: "summary".to_string(), field: new.into() },
                        };

                        bot_modules_auditlogs::events::dispatch_audit_log(
                            ctx,
                            data,
                            "AR/Inspector_MemberUpdateHoistAttempt",
                            "(Anti-Raid) Member Update Hoist Attempt",
                            imap,
                            ectx.guild_id,
                        )
                        .await?;
                    }
                }
            }

            Ok(())
        }
        _ => Ok(()),
    }
}
