use poise::serenity_prelude::FullEvent;
use silverpelt::ar_event::{AntiraidEvent, EventHandlerContext};

#[inline]
pub(crate) const fn not_audit_loggable_event() -> &'static [&'static str] {
    &[
        "CACHE_READY",         // Internal
        "INTERACTION_CREATE",  // Spams too much / is useless
        "RATELIMIT",           // Internal
        "GUILD_CREATE",        // Internal
        "GUILD_MEMBERS_CHUNK", // Internal
    ]
}

pub(crate) async fn event_listener(ectx: &EventHandlerContext) -> Result<(), silverpelt::Error> {
    let ctx = &ectx.serenity_context;

    match ectx.event {
        AntiraidEvent::Discord(ref event) => {
            if not_audit_loggable_event().contains(&event.into()) {
                return Ok(());
            }

            let user_id = gwevent::core::get_event_user_id(event);

            // Ignore ourselves
            match event {
                FullEvent::GuildAuditLogEntryCreate { .. } => {}
                _ => match user_id {
                    Some(user_id) => {
                        if user_id == ctx.cache.current_user().id {
                            return Ok(());
                        }
                    }
                    None => {}
                },
            }

            // Convert to titlecase by capitalizing the first letter of each word
            let event_titlename = event
                .snake_case_name()
                .split('_')
                .map(|s| {
                    let mut c = s.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().chain(c).collect(),
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");

            dispatch(
                ctx,
                &ectx.data,
                templating::event::CreateEventArc::new_arc(
                    event_titlename,
                    "Discord".to_string(),
                    event.snake_case_name().to_uppercase(),
                    serde_json::to_value(event)?.into(),
                    false,
                    user_id.map(|u| u.to_string()),
                ),
                ectx.guild_id,
            )
            .await
        }

        AntiraidEvent::Custom(ref event) => {
            dispatch(
                ctx,
                &ectx.data,
                templating::event::CreateEventArc::new(
                    event.event_titlename.clone(),
                    "Custom".to_string(),
                    event.event_name.clone(),
                    event.event_data.clone(),
                    false,
                    None,
                ),
                ectx.guild_id,
            )
            .await
        }
        AntiraidEvent::StingCreate(ref sting) => {
            dispatch(
                ctx,
                &ectx.data,
                templating::event::CreateEventArc::new_arc(
                    "(Anti Raid) Sting Created".to_string(),
                    "StingCreate".to_string(),
                    "StingCreate".to_string(),
                    serde_json::to_value(&sting)?.into(),
                    false,
                    None,
                ),
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::StingExpire(ref sting) => {
            dispatch(
                ctx,
                &ectx.data,
                templating::event::CreateEventArc::new_arc(
                    "(Anti Raid) Sting Expired".to_string(),
                    "StingExpire".to_string(),
                    "StingExpire".to_string(),
                    serde_json::to_value(&sting)?.into(),
                    false,
                    None,
                ),
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::StingDelete(ref sting) => {
            dispatch(
                ctx,
                &ectx.data,
                templating::event::CreateEventArc::new_arc(
                    "(Anti Raid) Sting Deleted".to_string(),
                    "StingDelete".to_string(),
                    "StingDelete".to_string(),
                    serde_json::to_value(&sting)?.into(),
                    false,
                    None,
                ),
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::PunishmentCreate(ref punishment) => {
            dispatch(
                ctx,
                &ectx.data,
                templating::event::CreateEventArc::new_arc(
                    "(Anti Raid) Punishment Created".to_string(),
                    "PunishmentCreate".to_string(),
                    "PunishmentCreate".to_string(),
                    serde_json::to_value(&punishment)?.into(),
                    false,
                    None,
                ),
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::PunishmentExpire(ref punishment) => {
            dispatch(
                ctx,
                &ectx.data,
                templating::event::CreateEventArc::new_arc(
                    "(Anti Raid) Punishment Expired".to_string(),
                    "PunishmentExpire".to_string(),
                    "PunishmentExpire".to_string(),
                    serde_json::to_value(&punishment)?.into(),
                    false,
                    None,
                ),
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::OnStartup(ref modified) => {
            dispatch(
                ctx,
                &ectx.data,
                templating::event::CreateEventArc::new(
                    "(Anti Raid) On Startup".to_string(),
                    "OnStartup".to_string(),
                    "OnStartup".to_string(),
                    serde_json::json!({
                        "targets": modified
                    }),
                    false,
                    None,
                ),
                ectx.guild_id,
            )
            .await
        }
    }
}

/// Check if an event matches a list of filters
///
/// Rules:
/// - If filter is empty, return true unless a special case applies
/// - If filter matches the event_name, return true unless a special case applies
///
/// Special cases:
/// - If event_name is MESSAGE, then it must be an exact match to be dispatched AND must have a custom template declared for it. This is to avoid spam
pub(crate) async fn should_dispatch_event(
    event_name: &str,
    filters: &[String],
) -> Result<bool, silverpelt::Error> {
    if event_name == "MESSAGE" || event_name == "AR/CheckCommand" || event_name == "AR/OnStartup" {
        // Message should only be fired if the template explicitly wants the event
        if !filters.contains(&event_name.to_string()) {
            return Ok(false);
        }

        return Ok(true);
    }

    // If empty, always return Ok
    if filters.is_empty() {
        return Ok(true);
    }

    Ok(filters.contains(&event_name.to_string()))
}

async fn dispatch(
    ctx: &serenity::all::client::Context,
    data: &silverpelt::data::Data,
    event: templating::event::CreateEventArc,
    guild_id: serenity::model::id::GuildId,
) -> Result<(), silverpelt::Error> {
    let templates = templating::cache::get_all_guild_templates(guild_id, &data.pool).await?;

    if templates.is_empty() {
        return Ok(());
    }

    for template in templates.iter() {
        // Verify event dispatch
        if !should_dispatch_event(&event.name, {
            // False positive, unwrap_or_default cannot be used here as it moves the event out of the sink
            #[allow(clippy::manual_unwrap_or_default)]
            if let Some(ref events) = template.events {
                events
            } else {
                &[]
            }
        })
        .await?
        {
            continue;
        }

        match templating::execute::<Option<()>>(
            guild_id,
            templating::Template::Named(template.name.clone()),
            data.pool.clone(),
            ctx.clone(),
            data.reqwest.clone(),
            event.into_event(),
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                dispatch_error(ctx, data, &e.to_string(), guild_id, template).await?;
            }
        }
    }

    Ok(())
}

/// Dispatches an error event
async fn dispatch_error(
    ctx: &serenity::all::client::Context,
    data: &silverpelt::data::Data,
    error: &str,
    guild_id: serenity::model::id::GuildId,
    template: &templating::GuildTemplate,
) -> Result<(), silverpelt::Error> {
    let templates = templating::cache::get_all_guild_templates(guild_id, &data.pool).await?;

    if templates.is_empty() {
        return Ok(());
    }

    match template.error_channel {
        Some(c) => {
            let Some(channel) = sandwich_driver::channel(
                &botox::cache::CacheHttpImpl::from_ctx(ctx),
                &data.reqwest,
                Some(guild_id),
                c,
            )
            .await?
            else {
                return Ok(());
            };

            let Some(guild_channel) = channel.guild() else {
                return Ok(());
            };

            if guild_channel.guild_id != guild_id {
                return Ok(());
            }

            c.send_message(
                &ctx.http,
                serenity::all::CreateMessage::new()
                    .embed(
                        serenity::all::CreateEmbed::new()
                            .title("Error executing template")
                            .field("Error", error, false)
                            .field("Template", template.name.clone(), false),
                    )
                    .components(vec![serenity::all::CreateActionRow::Buttons(
                        vec![serenity::all::CreateButton::new_link(
                            &config::CONFIG.meta.support_server_invite,
                        )
                        .label("Support Server")]
                        .into(),
                    )]),
            )
            .await?;
        }
        None => {
            // Try firing the error event
            templating::execute::<Option<()>>(
                guild_id,
                templating::Template::Named(template.name.clone()),
                data.pool.clone(),
                ctx.clone(),
                data.reqwest.clone(),
                templating::event::CreateEventArc::new(
                    "Error".to_string(),
                    "Error".to_string(),
                    "Error".to_string(),
                    error.into(),
                    false,
                    None,
                )
                .into_event(),
            )
            .await?;
        }
    }

    Ok(())
}
