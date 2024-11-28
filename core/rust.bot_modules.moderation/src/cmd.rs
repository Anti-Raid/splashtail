use futures_util::StreamExt;
use jobserver::embed::{embed as embed_job, get_icon_of_state};
use poise::CreateReply;
use sandwich_driver::{guild, member_in_guild};
use serenity::all::{
    ChannelId, CreateEmbed, EditMember, EditMessage, GuildId, Mentionable, Timestamp, User, UserId,
};
use silverpelt::Context;
use silverpelt::Error;
use splashcore_rs::utils::{
    create_special_allocation_from_str, parse_duration_string, parse_numeric_list_to_str, Unit,
    REPLACE_CHANNEL,
};
use std::collections::HashMap;

/// Helper method to get the username of a user
pub fn username(m: &User) -> String {
    if let Some(ref global_name) = m.global_name {
        global_name.to_string()
    } else {
        m.tag()
    }
}

/// Helper method to get the username of a member
pub fn to_log_format(moderator: &User, member: &User, reason: &str) -> String {
    format!(
        "{} | Handled '{}' for reason '{}'",
        username(moderator),
        username(member),
        reason
    )
}

/*
// Options that can be set when pruning a message
//
// Either one of PruneFrom or MaxMessages must be set. If both are set, then both will be used.
type MessagePruneOpts struct {
    UserID             string         `description:"If set, the user id to prune messages of"`
    Channels           []string       `description:"If set, the channels to prune messages from"`
    IgnoreErrors       bool           `description:"If set, ignore errors while pruning"`
    MaxMessages        int            `description:"The maximum number of messages to prune"`
    PruneFrom          timex.Duration `description:"If set, the time to prune messages from."`
    PerChannel         int            `description:"The minimum number of messages to prune per channel"`
    RolloverLeftovers  bool           `description:"Whether to attempt rollover of leftover message quota to another channels or not"`
    SpecialAllocations map[string]int `description:"Specific channel allocation overrides"`
}
*/

#[allow(clippy::too_many_arguments)]
fn create_message_prune_serde(
    user_id: UserId,
    guild_id: GuildId,
    channels: &Option<String>,
    ignore_errors: Option<bool>,
    max_messages: Option<i32>,
    prune_from: Option<String>,
    per_channel: Option<i32>,
    rollover_leftovers: Option<bool>,
    special_allocations: Option<String>,
) -> Result<serde_json::Value, Error> {
    let channels = if let Some(ref channels) = channels {
        parse_numeric_list_to_str::<ChannelId>(channels, &REPLACE_CHANNEL)?
    } else {
        vec![]
    };

    let prune_from = if let Some(ref prune_from) = prune_from {
        let (dur, unit) = parse_duration_string(prune_from)?;

        dur * unit.to_seconds()
    } else {
        0
    };

    let special_allocations = if let Some(ref special_allocations) = special_allocations {
        create_special_allocation_from_str(special_allocations)?
    } else {
        HashMap::new()
    };

    Ok(serde_json::json!(
        {
            "ServerID": guild_id.to_string(),
            "Options": {
                "UserID": user_id.to_string(),
                "Channels": channels,
                "IgnoreErrors": ignore_errors.unwrap_or(false),
                "MaxMessages": max_messages.unwrap_or(1000),
                "PruneFrom": prune_from,
                "PerChannel": per_channel.unwrap_or(100),
                "RolloverLeftovers": rollover_leftovers.unwrap_or(false),
                "SpecialAllocations": special_allocations,
            }
        }
    ))
}

/// Helper method to check the author of a user versus a target
async fn check_hierarchy(ctx: &Context<'_>, user_id: UserId) -> Result<(), Error> {
    let data = ctx.data();
    let sctx = ctx.serenity_context();
    let cache_http = botox::cache::CacheHttpImpl::from_ctx(sctx);

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let guild = guild(&cache_http, &data.reqwest, guild_id).await?;

    let author_id = ctx.author().id;

    let bot_userid = sctx.cache.current_user().id;
    let Some(bot) = member_in_guild(&cache_http, &data.reqwest, guild_id, bot_userid).await? else {
        return Err("Bot member not found".into());
    };

    let Some(author) = member_in_guild(&cache_http, &data.reqwest, guild_id, author_id).await?
    else {
        return Err("Message author not found".into());
    };

    let Some(user) = member_in_guild(&cache_http, &data.reqwest, guild_id, user_id).await? else {
        // User is not in the server, so yes, they're below us
        return Ok(());
    };

    if let Some(higher_hierarchy) = guild.greater_member_hierarchy(&bot, &user) {
        if higher_hierarchy != bot_userid {
            log::info!("Roles of lhs: {:?}", bot.roles);
            log::info!("Roles of rhs: {:?}", user.roles);
            return Err(format!("You cannot moderate a user with a higher or equal hierarchy to the bot ({} has higher hierarchy)", higher_hierarchy.mention()).into());
        }
    } else {
        return Err("You cannot moderate a user with equal hierarchy to the bot".into());
    }

    if let Some(higher_hierarchy) = guild.greater_member_hierarchy(&author, &user) {
        if higher_hierarchy != author_id {
            Err("You cannot moderate a user with a higher or equal hierarchy than you".into())
        } else {
            Ok(())
        }
    } else {
        Err("You cannot moderate a user with equal hierarchy to you".into())
    }
}

/// Prune messages from a user
#[poise::command(
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "KICK_MEMBERS | MANAGE_MESSAGES"
)]
#[allow(clippy::too_many_arguments)]
pub async fn prune_user(
    ctx: Context<'_>,
    #[description = "The user to prune"] user: serenity::all::User,
    #[description = "The reason for the prune"]
    #[max_length = 512]
    reason: String,
    #[description = "Number of stings to give. Defaults to configured base stings"] stings: Option<
        i32,
    >,
    #[description = "Whether or not to show prune status updates"] prune_debug: Option<bool>,
    #[description = "Channels to prune from, otherwise will prune from all channels"]
    prune_channels: Option<String>,
    #[description = "Whether or not to avoid errors while pruning"] prune_ignore_errors: Option<
        bool,
    >,
    #[description = "How many messages at maximum to prune"] prune_max_messages: Option<i32>,
    #[description = "The duration to prune from. Format: <number> days/hours/minutes/seconds"]
    prune_from: Option<String>,
    #[description = "The minimum number of messages to prune per channel"]
    prune_per_channel: Option<i32>,
    #[description = "Whether to attempt rollover of leftover message quota to another channels or not"]
    prune_rollover_leftovers: Option<bool>,
    #[description = "Specific channel allocation overrides"] prune_special_allocations: Option<
        String,
    >,
) -> Result<(), Error> {
    if reason.len() > 512 {
        return Err("Reason must be less than/equal to 512 characters".into());
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    // Check user hierarchy before performing moderative actions
    check_hierarchy(&ctx, user.id).await?;

    let mut embed = CreateEmbed::new()
        .title("Pruning User Messages...")
        .description(format!(
            "{} | Pruning User Messages {}",
            get_icon_of_state("pending"),
            user.mention()
        ));

    let base_message = ctx.send(CreateReply::new().embed(embed)).await?;

    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    let stings = stings.unwrap_or(0);

    if stings < 0 {
        return Err("Stings must be greater than or equal to 0".into());
    }

    let mut tx = ctx.data().pool.begin().await?;

    let mut sting_dispatch = None;

    if stings > 0 {
        sting_dispatch = Some(
            silverpelt::stings::StingCreate {
                module: "moderation".to_string(),
                src: Some("prune_user".to_string()),
                stings,
                reason: Some(reason.clone()),
                void_reason: None,
                guild_id,
                creator: silverpelt::stings::StingTarget::User(author.user.id),
                target: silverpelt::stings::StingTarget::User(user.id),
                state: silverpelt::stings::StingState::Active,
                duration: None,
                sting_data: None,
            }
            .create_without_dispatch(&mut *tx)
            .await?,
        );
    }

    // If we're pruning messages, do that
    let prune_opts = create_message_prune_serde(
        user.id,
        guild_id,
        &prune_channels,
        prune_ignore_errors,
        prune_max_messages,
        prune_from,
        prune_per_channel,
        prune_rollover_leftovers,
        prune_special_allocations,
    )?;

    let data = ctx.data();

    // Make request to jobserver
    let id = jobserver::spawn::spawn_task(
        &data.reqwest,
        &jobserver::Spawn {
            name: "message_prune".to_string(),
            data: prune_opts.clone(),
            create: true,
            execute: true,
            id: None,
            user_id: author.user.id.to_string(),
        },
    )
    .await?
    .id;

    tx.commit().await?;

    if let Some(sting_dispatch) = sting_dispatch {
        sting_dispatch
            .dispatch_create_event(ctx.serenity_context().clone())
            .await?;
    };

    silverpelt::ar_event::dispatch_event_to_modules(&silverpelt::ar_event::EventHandlerContext {
        guild_id,
        data,
        event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
            event_name: "AR/PruneUser".to_string(),
            event_titlename: "(Anti-Raid) Prune User".to_string(),
            event_data: serde_json::json!({
                "log": to_log_format(&author.user, &user, &reason),
                "prune_opts": prune_opts,
                "channels": if let Some(ref channels) = prune_channels {
                    parse_numeric_list_to_str::<ChannelId>(channels, &REPLACE_CHANNEL)?
                } else {
                    Vec::new()
                },
            }),
        }),
        serenity_context: ctx.serenity_context().clone(),
    })
    .await
    .map_err(|e| {
        format!("Failed to dispatch event: {}", {
            let mut strs = String::new();

            for err in e {
                strs.push_str(&format!("{}\n", err));
            }

            strs
        })
    })?;

    embed = CreateEmbed::new()
        .title("Pruning User Messages...")
        .description(format!(
            "{} | Pruning User Messages...",
            get_icon_of_state("pending")
        ))
        .field(
            "Pruning Messages",
            format!(":yellow_circle: Created job with ID of {}", id),
            false,
        );

    base_message
        .edit(ctx.clone(), CreateReply::new().embed(embed.clone()))
        .await?;

    let mut stream = Box::pin(jobserver::poll::reactive(
        &ctx.data().pool,
        &id,
        jobserver::poll::PollTaskOptions::default(),
    )?);

    while let Some(job) = stream.next().await {
        match job {
            Ok(Some(job)) => {
                let new_job_msg = embed_job(
                    &config::CONFIG.sites.api,
                    &job,
                    vec![CreateEmbed::default()
                        .title("Pruning User Messages...")
                        .description(format!(
                            "{} | Pruning User Messages {}",
                            get_icon_of_state(&job.state),
                            user.mention()
                        ))],
                    prune_debug.unwrap_or(false),
                )?;

                base_message.edit(ctx.clone(), new_job_msg).await?;
            }
            Ok(None) => {
                continue; // Go to the next iteration
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(())
}

/// Kicks a member from the server with optional purge/stinging abilities
#[poise::command(
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "KICK_MEMBERS | MANAGE_MESSAGES"
)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "The member to kick"] member: serenity::all::Member,
    #[description = "The reason for the kick"]
    #[max_length = 384]
    reason: String,
    #[description = "Number of stings to give. Defaults to configured base stings"] stings: Option<
        i32,
    >,
) -> Result<(), Error> {
    if reason.len() > 384 {
        return Err("Reason must be less than/equal to 384 characters".into());
    }

    let stings = stings.unwrap_or(0);

    if stings < 0 {
        return Err("Stings must be greater than or equal to 0".into());
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let data = ctx.data();

    // Check user hierarchy before performing moderative actions
    check_hierarchy(&ctx, member.user.id).await?;

    // Dispatch event to modules, erroring out if the dispatch errors (e.g. limits hit due to a lua template etc)
    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    silverpelt::ar_event::dispatch_event_to_modules(&silverpelt::ar_event::EventHandlerContext {
        guild_id,
        data: data.clone(),
        event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
            event_name: "AR/KickMember/Start".to_string(),
            event_titlename: "(Anti-Raid) Kick Member (Pre-Warning)".to_string(),
            event_data: serde_json::json!({
                "target": member.clone(),
                "moderator": author,
                "reason": reason.clone(),
                "stings": stings,
                "log": to_log_format(&author.user, &member.user, &reason),
            }),
        }),
        serenity_context: ctx.serenity_context().clone(),
    })
    .await
    .map_err(|e| {
        format!("Failed to dispatch event: {}", {
            let mut strs = String::new();

            for err in e {
                strs.push_str(&format!("{}\n", err));
            }

            strs
        })
    })?;

    let mut embed = CreateEmbed::new()
        .title("Kicking Member...")
        .description(format!(
            "{} | Kicking {}",
            get_icon_of_state("pending"),
            member.mention()
        ));

    let mut base_message = ctx
        .send(CreateReply::new().embed(embed))
        .await?
        .into_message()
        .await?;

    // Try kicking them
    let mut tx = data.pool.begin().await?;

    let mut sting_dispatch = None;

    if stings > 0 {
        sting_dispatch = Some(
            silverpelt::stings::StingCreate {
                module: "moderation".to_string(),
                src: Some("kick".to_string()),
                stings,
                reason: Some(reason.clone()),
                void_reason: None,
                guild_id,
                creator: silverpelt::stings::StingTarget::User(author.user.id),
                target: silverpelt::stings::StingTarget::User(member.user.id),
                state: silverpelt::stings::StingState::Active,
                duration: None,
                sting_data: None,
            }
            .create_without_dispatch(&mut *tx)
            .await?,
        );
    }

    // Create new punishment
    let p = silverpelt::punishments::PunishmentCreate {
        module: "moderation".to_string(),
        src: Some("kick".to_string()),
        guild_id,
        punishment: "kick".to_string(),
        creator: silverpelt::punishments::PunishmentTarget::User(author.user.id),
        target: silverpelt::punishments::PunishmentTarget::User(member.user.id),
        handle_log: serde_json::json!({}),
        duration: None,
        reason: reason.clone(),
        data: None,
    }
    .create_without_dispatch(&mut *tx)
    .await?;

    member
        .kick(
            ctx.http(),
            Some(&to_log_format(&author.user, &member.user, &reason)),
        )
        .await?;

    tx.commit().await?;

    silverpelt::ar_event::dispatch_event_to_modules(&silverpelt::ar_event::EventHandlerContext {
        guild_id,
        data: data.clone(),
        event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
            event_name: "AR/KickMember/End".to_string(),
            event_titlename: "(Anti-Raid) Kick Member (Post-Warning)".to_string(),
            event_data: serde_json::json!({
                "target": member,
                "moderator": author,
                "reason": reason.clone(),
                "stings": stings,
                "log": to_log_format(&author.user, &member.user, &reason),
            }),
        }),
        serenity_context: ctx.serenity_context().clone(),
    })
    .await
    .map_err(|e| {
        format!("Failed to dispatch event: {}", {
            let mut strs = String::new();

            for err in e {
                strs.push_str(&format!("{}\n", err));
            }

            strs
        })
    })?;

    p.dispatch_event(ctx.serenity_context().clone()).await?;
    if let Some(sting_dispatch) = sting_dispatch {
        sting_dispatch
            .dispatch_create_event(ctx.serenity_context().clone())
            .await?;
    };

    embed = CreateEmbed::new()
        .title("Kicking Member...")
        .description(format!(
            "{} | Kicked {}",
            get_icon_of_state("completed"),
            member.mention()
        ));

    base_message
        .edit(&ctx.http(), EditMessage::new().embed(embed))
        .await?;

    Ok(())
}

/// Bans a member from the server with optional purge/stinging abilities
#[poise::command(
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "BAN_MEMBERS | MANAGE_MESSAGES"
)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "The member to ban"] member: serenity::all::User,
    #[description = "The reason for the ban"]
    #[max_length = 384]
    reason: String,
    #[description = "Number of stings to give. Defaults to configured base stings"] stings: Option<
        i32,
    >,
    #[description = "How many messages to prune using discords autopruner [dmd] (days)"] prune_dmd: Option<u8>,
) -> Result<(), Error> {
    if reason.len() > 384 {
        return Err("Reason must be less than/equal to 384 characters".into());
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let stings = stings.unwrap_or(1);

    if stings < 0 {
        return Err("Stings must be greater than or equal to 0".into());
    }

    let dmd = prune_dmd.unwrap_or_default();

    let data = ctx.data();

    // Check user hierarchy before performing moderative actions
    check_hierarchy(&ctx, member.id).await?;

    // Dispatch event to modules, erroring out if the dispatch errors (e.g. limits hit due to a lua template etc)
    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    silverpelt::ar_event::dispatch_event_to_modules(&silverpelt::ar_event::EventHandlerContext {
        guild_id,
        data: data.clone(),
        event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
            event_name: "AR/BanMember/Start".to_string(),
            event_titlename: "(Anti-Raid) Ban Member (Pre-Warning)".to_string(),
            event_data: serde_json::json!({
                "target": member,
                "moderator": author,
                "reason": reason,
                "stings": stings,
                "prune_dmd": dmd,
                "log": to_log_format(&author.user, &member, &reason),
            }),
        }),
        serenity_context: ctx.serenity_context().clone(),
    })
    .await
    .map_err(|e| {
        format!("Failed to dispatch event: {}", {
            let mut strs = String::new();

            for err in e {
                strs.push_str(&format!("{}\n", err));
            }

            strs
        })
    })?;

    let mut embed = CreateEmbed::new()
        .title("Banning Member...")
        .description(format!(
            "{} | Banning {}",
            get_icon_of_state("pending"),
            member.mention()
        ));

    let mut base_message = ctx
        .send(CreateReply::new().embed(embed))
        .await?
        .into_message()
        .await?;

    let mut tx = data.pool.begin().await?;

    let mut sting_dispatch = None;

    if stings > 0 {
        sting_dispatch = Some(
            silverpelt::stings::StingCreate {
                module: "moderation".to_string(),
                src: Some("ban".to_string()),
                stings,
                reason: Some(reason.clone()),
                void_reason: None,
                guild_id,
                creator: silverpelt::stings::StingTarget::User(author.user.id),
                target: silverpelt::stings::StingTarget::User(member.id),
                state: silverpelt::stings::StingState::Active,
                duration: None,
                sting_data: None,
            }
            .create_without_dispatch(&mut *tx)
            .await?,
        );
    }

    // Create new punishment
    let p = silverpelt::punishments::PunishmentCreate {
        module: "moderation".to_string(),
        src: Some("ban".to_string()),
        guild_id,
        punishment: "ban".to_string(),
        creator: silverpelt::punishments::PunishmentTarget::User(author.user.id),
        target: silverpelt::punishments::PunishmentTarget::User(member.id),
        handle_log: serde_json::json!({}),
        duration: None,
        reason: reason.clone(),
        data: None,
    }
    .create_without_dispatch(&mut *tx)
    .await?;

    guild_id
        .ban(
            ctx.http(),
            member.id,
            dmd,
            Some(&to_log_format(&author.user, &member, &reason)),
        )
        .await?;

    tx.commit().await?;

    silverpelt::ar_event::dispatch_event_to_modules(&silverpelt::ar_event::EventHandlerContext {
        guild_id,
        data: data.clone(),
        event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
            event_name: "AR/BanMember/End".to_string(),
            event_titlename: "(Anti-Raid) Ban Member (Post-Warning)".to_string(),
            event_data: serde_json::json!({
                "target": member,
                "moderator": author,
                "reason": reason,
                "stings": stings,
                "prune_dmd": dmd,
                "log": to_log_format(&author.user, &member, &reason),
            }),
        }),
        serenity_context: ctx.serenity_context().clone(),
    })
    .await
    .map_err(|e| {
        format!("Failed to dispatch event: {}", {
            let mut strs = String::new();

            for err in e {
                strs.push_str(&format!("{}\n", err));
            }

            strs
        })
    })?;

    p.dispatch_event(ctx.serenity_context().clone()).await?;
    if let Some(sting_dispatch) = sting_dispatch {
        sting_dispatch
            .dispatch_create_event(ctx.serenity_context().clone())
            .await?;
    };

    embed = CreateEmbed::new()
        .title("Banning Member...")
        .description(format!(
            "{} | Banned {}",
            get_icon_of_state("completed"),
            member.mention()
        ));

    base_message
        .edit(&ctx.http(), EditMessage::new().embed(embed))
        .await?;

    Ok(())
}

/// Temporaily bans a member from the server with optional purge/stinging abilities
#[poise::command(
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "BAN_MEMBERS | MANAGE_MESSAGES"
)]
pub async fn tempban(
    ctx: Context<'_>,
    #[description = "The member to ban"] member: serenity::all::User,
    #[description = "The reason for the ban"]
    #[max_length = 384]
    reason: String,
    #[description = "Number of stings to give. Defaults to configured base stings"] stings: Option<
        i32,
    >,
    #[description = "The duration of the ban"] duration: String,
    #[description = "How many messages to prune using discords autopruner [dmd] (days)"] prune_dmd: Option<u8>,
) -> Result<(), Error> {
    if reason.len() > 384 {
        return Err("Reason must be less than/equal to 384 characters".into());
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let stings = stings.unwrap_or(1);

    if stings < 0 {
        return Err("Stings must be greater than or equal to 0".into());
    }

    let dmd = prune_dmd.unwrap_or_default();

    let duration = parse_duration_string(&duration)?;

    let data = ctx.data();

    // Check user hierarchy before performing moderative actions
    check_hierarchy(&ctx, member.id).await?;

    // Dispatch event to modules, erroring out if the dispatch errors (e.g. limits hit due to a lua template etc)
    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    silverpelt::ar_event::dispatch_event_to_modules(&silverpelt::ar_event::EventHandlerContext {
        guild_id,
        data: data.clone(),
        event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
            event_name: "AR/BanMemberTemporary/Start".to_string(),
            event_titlename: "(Anti-Raid) Ban Member (Temporary) (Pre-Warning)".to_string(),
            event_data: serde_json::json!({
                "target": member,
                "moderator": author,
                "reason": reason,
                "stings": stings,
                "prune_dmd": dmd,
                "log": to_log_format(&author.user, &member, &reason),
                "duration": (duration.0 * duration.1.to_seconds()),
            }),
        }),
        serenity_context: ctx.serenity_context().clone(),
    })
    .await
    .map_err(|e| {
        format!("Failed to dispatch event: {}", {
            let mut strs = String::new();

            for err in e {
                strs.push_str(&format!("{}\n", err));
            }

            strs
        })
    })?;

    let mut embed = CreateEmbed::new()
        .title("(Temporarily) Banning Member...")
        .description(format!(
            "{} | Banning {}",
            get_icon_of_state("pending"),
            member.mention()
        ));

    let mut base_message = ctx
        .send(CreateReply::new().embed(embed))
        .await?
        .into_message()
        .await?;

    let mut tx = data.pool.begin().await?;

    let mut sting_dispatch = None;

    if stings > 0 {
        sting_dispatch = Some(
            silverpelt::stings::StingCreate {
                module: "moderation".to_string(),
                src: Some("tempban".to_string()),
                stings,
                reason: Some(reason.clone()),
                void_reason: None,
                guild_id,
                creator: silverpelt::stings::StingTarget::User(author.user.id),
                target: silverpelt::stings::StingTarget::User(member.id),
                state: silverpelt::stings::StingState::Active,
                duration: Some(std::time::Duration::from_secs(
                    duration.0 * duration.1.to_seconds(),
                )),
                sting_data: None,
            }
            .create_without_dispatch(&mut *tx)
            .await?,
        );
    }

    // Create new punishment
    let p = silverpelt::punishments::PunishmentCreate {
        module: "moderation".to_string(),
        src: Some("tempban".to_string()),
        guild_id,
        punishment: "ban".to_string(),
        creator: silverpelt::punishments::PunishmentTarget::User(author.user.id),
        target: silverpelt::punishments::PunishmentTarget::User(member.id),
        handle_log: serde_json::json!({}),
        duration: Some(std::time::Duration::from_secs(
            duration.0 * duration.1.to_seconds(),
        )),
        reason: reason.clone(),
        data: None,
    }
    .create_without_dispatch(&mut *tx)
    .await?;

    guild_id
        .ban(
            ctx.http(),
            member.id,
            dmd,
            Some(&to_log_format(&author.user, &member, &reason)),
        )
        .await?;

    tx.commit().await?;

    silverpelt::ar_event::dispatch_event_to_modules(&silverpelt::ar_event::EventHandlerContext {
        guild_id,
        data: data.clone(),
        event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
            event_name: "AR/BanMemberTemporary/End".to_string(),
            event_titlename: "(Anti-Raid) Ban Member (Temporary) (Post-Warning)".to_string(),
            event_data: serde_json::json!({
                "target": member,
                "moderator": author,
                "reason": reason,
                "stings": stings,
                "prune_dmd": dmd,
                "log": to_log_format(&author.user, &member, &reason),
                "duration": (duration.0 * duration.1.to_seconds()),
            }),
        }),
        serenity_context: ctx.serenity_context().clone(),
    })
    .await
    .map_err(|e| {
        format!("Failed to dispatch event: {}", {
            let mut strs = String::new();

            for err in e {
                strs.push_str(&format!("{}\n", err));
            }

            strs
        })
    })?;

    p.dispatch_event(ctx.serenity_context().clone()).await?;
    if let Some(sting_dispatch) = sting_dispatch {
        sting_dispatch
            .dispatch_create_event(ctx.serenity_context().clone())
            .await?;
    };

    embed = CreateEmbed::new()
        .title("(Temporarily) Banned Member...")
        .description(format!(
            "{} | Banned {}",
            get_icon_of_state("completed"),
            member.mention()
        ));

    base_message
        .edit(&ctx.http(), EditMessage::new().embed(embed))
        .await?;

    Ok(())
}

/// Unbans a member from the server with optional purge/stinging abilities
#[poise::command(
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "BAN_MEMBERS | MANAGE_MESSAGES"
)]
pub async fn unban(
    ctx: Context<'_>,
    #[description = "The user to unban"] user: serenity::all::User,
    #[description = "The reason/justification for unbanning"]
    #[max_length = 384]
    reason: String,
    #[description = "Number of stings to give. Defaults to 0"] stings: Option<i32>,
) -> Result<(), Error> {
    if reason.len() > 384 {
        return Err("Reason must be less than/equal to 384 characters".into());
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let stings = stings.unwrap_or(1);

    if stings < 0 {
        return Err("Stings must be greater than or equal to 0".into());
    }

    let data = ctx.data();

    // Dispatch event to modules, erroring out if the dispatch errors (e.g. limits hit due to a lua template etc)
    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    silverpelt::ar_event::dispatch_event_to_modules(&silverpelt::ar_event::EventHandlerContext {
        guild_id,
        data: data.clone(),
        event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
            event_name: "AR/UnbanMember/Start".to_string(),
            event_titlename: "(Anti-Raid) Unban Member (Pre-Warning)".to_string(),
            event_data: serde_json::json!({
                "target": user,
                "moderator": author,
                "reason": reason,
                "stings": stings,
                "log": to_log_format(&author.user, &user, &reason),
            }),
        }),
        serenity_context: ctx.serenity_context().clone(),
    })
    .await
    .map_err(|e| {
        format!("Failed to dispatch event: {}", {
            let mut strs = String::new();

            for err in e {
                strs.push_str(&format!("{}\n", err));
            }

            strs
        })
    })?;

    let mut embed = CreateEmbed::new()
        .title("Unbanning Member...")
        .description(format!(
            "{} | Unbanning {}",
            get_icon_of_state("pending"),
            user.mention()
        ));

    let mut base_message = ctx
        .send(CreateReply::new().embed(embed))
        .await?
        .into_message()
        .await?;

    let mut tx = data.pool.begin().await?;

    let mut sting_dispatch = None;

    if stings > 0 {
        sting_dispatch = Some(
            silverpelt::stings::StingCreate {
                module: "moderation".to_string(),
                src: Some("unban".to_string()),
                stings,
                reason: Some(reason.clone()),
                void_reason: None,
                guild_id,
                creator: silverpelt::stings::StingTarget::User(author.user.id),
                target: silverpelt::stings::StingTarget::User(user.id),
                state: silverpelt::stings::StingState::Active,
                duration: None,
                sting_data: None,
            }
            .create_without_dispatch(&mut *tx)
            .await?,
        );
    }

    ctx.http()
        .remove_ban(
            guild_id,
            user.id,
            Some(&to_log_format(&author.user, &user, &reason)),
        )
        .await?;

    tx.commit().await?;

    silverpelt::ar_event::dispatch_event_to_modules(&silverpelt::ar_event::EventHandlerContext {
        guild_id,
        data: data.clone(),
        event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
            event_name: "AR/UnbanMember/End".to_string(),
            event_titlename: "(Anti-Raid) Unban Member (Post-Warning)".to_string(),
            event_data: serde_json::json!({
                "target": user,
                "moderator": author,
                "reason": reason,
                "stings": stings,
                "log": to_log_format(&author.user, &user, &reason),
            }),
        }),
        serenity_context: ctx.serenity_context().clone(),
    })
    .await
    .map_err(|e| {
        format!("Failed to dispatch event: {}", {
            let mut strs = String::new();

            for err in e {
                strs.push_str(&format!("{}\n", err));
            }

            strs
        })
    })?;

    if let Some(sting_dispatch) = sting_dispatch {
        sting_dispatch
            .dispatch_create_event(ctx.serenity_context().clone())
            .await?;
    };

    embed = CreateEmbed::new()
        .title("Unbanning Member...")
        .description(format!(
            "{} | Unbanned {}",
            get_icon_of_state("completed"),
            user.mention()
        ));

    base_message
        .edit(&ctx.http(), EditMessage::new().embed(embed))
        .await?;

    Ok(())
}

/// Times out a member from the server with optional purge/stinging abilities
#[poise::command(
    slash_command,
    guild_only,
    user_cooldown = "5",
    required_bot_permissions = "MODERATE_MEMBERS | MANAGE_MESSAGES"
)]
pub async fn timeout(
    ctx: Context<'_>,
    #[description = "The member to timeout"] mut member: serenity::all::Member,
    #[description = "The duration of the timeout"] duration: String,
    #[description = "The reason for the timeout"]
    #[max_length = 384]
    reason: String,
    #[description = "Number of stings to give. Defaults to configured base stings"] stings: Option<
        i32,
    >,
) -> Result<(), Error> {
    if reason.len() > 384 {
        return Err("Reason must be less than/equal to 384 characters".into());
    }

    let Some(guild_id) = ctx.guild_id() else {
        return Err("This command can only be used in a guild".into());
    };

    let data = ctx.data();

    // Try timing them out
    let duration = parse_duration_string(&duration)?;

    // Ensure less than 28 days = 4 weeks = 672 hours = 40320 minutes = 2419200 seconds
    if duration.0 > 7 && duration.1 == Unit::Weeks {
        return Err("Timeout duration must be less than 28 days (4 weeks)".into());
    } else if duration.0 > 28 && duration.1 == Unit::Days {
        return Err("Timeout duration must be less than 28 days".into());
    } else if duration.0 > 672 && duration.1 == Unit::Hours {
        return Err("Timeout duration must be less than 28 days (672 hours)".into());
    } else if duration.0 > 40320 && duration.1 == Unit::Minutes {
        return Err("Timeout duration must be less than 28 days (40320 minutes)".into());
    } else if duration.0 > 2419200 && duration.1 == Unit::Seconds {
        return Err("Timeout duration must be less than 28 days (2419200 seconds)".into());
    }

    let time = (duration.0 * duration.1.to_seconds() * 1000) as i64;

    let stings = stings.unwrap_or(1);

    if stings < 0 {
        return Err("Stings must be greater than or equal to 0".into());
    }

    // Check user hierarchy before performing moderative actions
    check_hierarchy(&ctx, member.user.id).await?;

    // Dispatch event to modules, erroring out if the dispatch errors (e.g. limits hit due to a lua template etc)
    let Some(author) = ctx.author_member().await else {
        return Err("This command can only be used in a guild".into());
    };

    silverpelt::ar_event::dispatch_event_to_modules(&silverpelt::ar_event::EventHandlerContext {
        guild_id,
        data: data.clone(),
        event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
            event_name: "AR/TimeoutMember/Start".to_string(),
            event_titlename: "(Anti-Raid) Timeout Member (Pre-Warning)".to_string(),
            event_data: serde_json::json!({
                "target": member,
                "moderator": author.user,
                "reason": reason,
                "stings": stings,
                "log": to_log_format(&author.user, &member.user, &reason),
                "duration": (duration.0 * duration.1.to_seconds()),
            }),
        }),
        serenity_context: ctx.serenity_context().clone(),
    })
    .await
    .map_err(|e| {
        format!("Failed to dispatch event: {}", {
            let mut strs = String::new();

            for err in e {
                strs.push_str(&format!("{}\n", err));
            }

            strs
        })
    })?;

    let mut embed = CreateEmbed::new()
        .title("Timing out Member...")
        .description(format!(
            "{} | Timing out {}",
            get_icon_of_state("pending"),
            member.mention()
        ));

    let mut base_message = ctx
        .send(CreateReply::new().embed(embed))
        .await?
        .into_message()
        .await?;

    let mut tx = data.pool.begin().await?;

    let mut sting_dispatch = None;

    if stings > 0 {
        sting_dispatch = Some(
            silverpelt::stings::StingCreate {
                module: "moderation".to_string(),
                src: Some("timeout".to_string()),
                stings,
                reason: Some(reason.clone()),
                void_reason: None,
                guild_id,
                creator: silverpelt::stings::StingTarget::User(author.user.id),
                target: silverpelt::stings::StingTarget::User(member.user.id),
                state: silverpelt::stings::StingState::Active,
                duration: Some(std::time::Duration::from_secs(
                    duration.0 * duration.1.to_seconds(),
                )),
                sting_data: None,
            }
            .create_without_dispatch(&mut *tx)
            .await?,
        );
    }

    // Create new punishment
    let p = silverpelt::punishments::PunishmentCreate {
        module: "moderation".to_string(),
        src: Some("timeout".to_string()),
        guild_id,
        punishment: "timeout".to_string(),
        creator: silverpelt::punishments::PunishmentTarget::User(author.user.id),
        target: silverpelt::punishments::PunishmentTarget::User(member.user.id),
        handle_log: serde_json::json!({}),
        duration: Some(std::time::Duration::from_secs(
            duration.0 * duration.1.to_seconds(),
        )),
        reason: reason.clone(),
        data: None,
    }
    .create_without_dispatch(&mut *tx)
    .await?;

    member
        .edit(
            ctx.http(),
            EditMember::new()
                .disable_communication_until(Timestamp::from_millis(
                    Timestamp::now().unix_timestamp() * 1000 + time,
                )?)
                .audit_log_reason(&to_log_format(&author.user, &member.user, &reason)),
        )
        .await?;

    tx.commit().await?;

    p.dispatch_event(ctx.serenity_context().clone()).await?;
    if let Some(sting_dispatch) = sting_dispatch {
        sting_dispatch
            .dispatch_create_event(ctx.serenity_context().clone())
            .await?;
    };

    silverpelt::ar_event::dispatch_event_to_modules(&silverpelt::ar_event::EventHandlerContext {
        guild_id,
        data: data.clone(),
        event: silverpelt::ar_event::AntiraidEvent::Custom(silverpelt::ar_event::CustomEvent {
            event_name: "AR/TimeoutMember/End".to_string(),
            event_titlename: "(Anti-Raid) Timeout Member (Post-Warning)".to_string(),
            event_data: serde_json::json!({
                "target": member,
                "moderator": author,
                "reason": reason,
                "stings": stings,
                "log": to_log_format(&author.user, &member.user, &reason),
                "duration": (duration.0 * duration.1.to_seconds()),
            }),
        }),
        serenity_context: ctx.serenity_context().clone(),
    })
    .await
    .map_err(|e| {
        format!("Failed to dispatch event: {}", {
            let mut strs = String::new();

            for err in e {
                strs.push_str(&format!("{}\n", err));
            }

            strs
        })
    })?;

    embed = CreateEmbed::new()
        .title("Timed Out Member...")
        .description(format!(
            "{} | Timing out {}",
            get_icon_of_state("completed"),
            member.mention()
        ));

    base_message
        .edit(&ctx.http(), EditMessage::new().embed(embed))
        .await?;

    Ok(())
}
