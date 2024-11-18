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
        silverpelt::ar_event::AntiraidEvent::TrustedWebEvent((ref event_name, ref data)) => {
            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/TrustedWebEvent",
                "(Anti Raid) Trusted Web Event",
                {
                    let mut m = serde_json::Map::new();
                    m.insert(
                        "event_name".to_string(),
                        serde_json::Value::String(event_name.to_string()),
                    );
                    m.insert("data".to_string(), data.clone());
                    serde_json::Value::Object(m)
                },
                ectx.guild_id,
            )
            .await
        }
        AntiraidEvent::Discord(ref event) => {
            if not_audit_loggable_event().contains(&event.into()) {
                return Ok(());
            }

            // Ignore ourselves
            match event {
                FullEvent::GuildAuditLogEntryCreate { .. } => {}
                _ => match gwevent::core::get_event_user_id(event) {
                    Ok(user_id) => {
                        if user_id == ctx.cache.current_user().id {
                            return Ok(());
                        }
                    }
                    Err(Some(e)) => {
                        return Err(e);
                    }
                    Err(None) => {}
                },
            }

            let event_data = serde_json::to_value(event)?;

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

            let event_name: &'static str = event.into();

            dispatch_audit_log(
                ctx,
                &ectx.data,
                event_name,
                &event_titlename,
                event_data,
                ectx.guild_id,
            )
            .await
        }

        AntiraidEvent::Custom(ref event) => {
            dispatch_audit_log(
                ctx,
                &ectx.data,
                &event.event_name,
                &event.event_titlename,
                event.event_data.clone(),
                ectx.guild_id,
            )
            .await
        }
        AntiraidEvent::StingCreate(ref sting) => {
            let sting = serde_json::to_value(sting)?;

            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/StingCreate",
                "(Anti Raid) Sting Created",
                serde_json::to_value(sting)?,
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::StingExpire(ref sting) => {
            let sting = serde_json::to_value(sting)?;

            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/StingExpire",
                "(Anti Raid) Sting Expired",
                serde_json::to_value(sting)?,
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::StingDelete(ref sting) => {
            let sting = serde_json::to_value(sting)?;

            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/StingDelete",
                "(Anti Raid) Sting Deleted",
                serde_json::to_value(sting)?,
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::PunishmentCreate(ref punishment) => {
            let punishment = serde_json::to_value(punishment)?;

            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/PunishmentCreate",
                "(Anti Raid) Punishment Created",
                serde_json::to_value(punishment)?,
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::PunishmentExpire(ref punishment) => {
            let punishment = serde_json::to_value(punishment)?;

            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/PunishmentExpire",
                "(Anti Raid) Punishment Expired",
                serde_json::to_value(punishment)?,
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::OnStartup(ref modified) => {
            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/OnStartup",
                "(Anti Raid) On Startup",
                serde_json::json!({
                    "targets": modified
                }),
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

async fn dispatch_audit_log(
    ctx: &serenity::all::client::Context,
    data: &silverpelt::data::Data,
    event_name: &str,
    event_titlename: &str,
    event_data: serde_json::Value,
    guild_id: serenity::model::id::GuildId,
) -> Result<(), silverpelt::Error> {
    let templates = templating::cache::get_all_guild_templates(guild_id, &data.pool).await?;

    if templates.is_empty() {
        return Ok(());
    }

    for template in templates.iter() {
        // Verify event dispatch
        if !should_dispatch_event(event_name, {
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

        templating::execute::<_, Option<()>>(
            guild_id,
            templating::Template::Named(template.name.clone()),
            data.pool.clone(),
            ctx.clone(),
            data.reqwest.clone(),
            HookContext {
                event_titlename: event_titlename.to_string(),
                event_name: event_name.to_string(),
                event_data: event_data.clone(),
                template: template.clone(),
            },
        )
        .await?;
    }

    Ok(())
}

/// A HookContext is a context for message templates
/// that can be accessed in hook templates
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct HookContext {
    pub event_titlename: String,
    pub event_name: String,
    pub event_data: serde_json::Value,
    pub template: templating::GuildTemplate,
}

#[typetag::serde]
impl templating::Context for HookContext {}
