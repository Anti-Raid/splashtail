use gwevent::field::Field;
use include_dir::{include_dir, Dir};
use log::warn;
use poise::serenity_prelude::FullEvent;
use serenity::all::{ChannelId, CreateMessage};
use silverpelt::ar_event::{AntiraidEvent, EventHandlerContext};
use std_events::auditlog::AuditLogDispatchEvent;

static DEFAULT_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/templates");

fn load_embedded_event_template(event: &str) -> Result<String, silverpelt::Error> {
    let template = match DEFAULT_TEMPLATES.get_file(format!("{}.art", event)) {
        Some(template) => template,
        None => {
            // Load default.art
            DEFAULT_TEMPLATES
                .get_file("default.luau")
                .ok_or("Failed to load default template")?
        }
    };

    let template_str = template.contents_utf8().ok_or("Failed to load template")?;

    Ok(template_str.to_string())
}

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
            if event_name != "checkAllEvents" {
                return Ok(()); // Ignore unknown events
            }

            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/TrustedWebEvent",
                "(Anti Raid) Trusted Web Event",
                indexmap::indexmap! {
                    "event_name".to_string() => event_name.clone().into(),
                    "data".to_string() => data.clone().into(),
                },
                ectx.guild_id,
            )
            .await
        }
        AntiraidEvent::Discord(ref event) => {
            if not_audit_loggable_event().contains(&event.into()) {
                return Ok(());
            }

            // (hopefully temporary) work around to reduce spam
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

            let Some(event_data) = gwevent::core::expand_event(event.clone()) else {
                // Event cannot be expanded, ignore
                return Ok(());
            };

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
            if event.target() == std_events::auditlog::AUDITLOG_TARGET_ID
                && event.event_name() == "AuditLog:DispatchEvent"
            {
                let Some(event) = event.as_any().downcast_ref::<AuditLogDispatchEvent>() else {
                    return Ok(()); // Ignore unknown events
                };

                dispatch_audit_log(
                    ctx,
                    &ectx.data,
                    &event.event_name,
                    &event.event_titlename,
                    event.event_data.clone(),
                    ectx.guild_id,
                )
                .await
            } else {
                Ok(())
            }
        }
        AntiraidEvent::StingCreate(ref sting) => {
            let sting_val = serde_json::to_value(sting)?;

            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/StingCreate",
                "(Anti Raid) Created Sting For User",
                indexmap::indexmap! {
                    "target".to_string() =>
                        match &sting.target {
                            silverpelt::stings::StingTarget::User(user_id) => (*user_id).into(),
                            silverpelt::stings::StingTarget::System => "System".to_string().into(),
                        },
                    "reason".to_string() =>
                        match &sting.reason {
                            Some(reason) => reason.clone().into(),
                            None => Field::None,
                        },
                    "stings".to_string() => sting.stings.into(),
                    "state".to_string() => sting.state.to_string().into(),
                    "sting_val".to_string() => sting_val.into(),
                },
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::PunishmentCreate(ref punishments) => {
            let punishment = serde_json::to_value(punishments)?;

            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/PunishmentCreate",
                "(Anti Raid) Created Punishment",
                indexmap::indexmap! {
                    "punishment".to_string() => punishment.into(),
                },
                ectx.guild_id,
            )
            .await?;

            Ok(())
        }
        AntiraidEvent::MemberVerify((user_id, ref data)) => {
            dispatch_audit_log(
                ctx,
                &ectx.data,
                "AR/MemberVerify",
                "(Anti Raid) Member Verify",
                indexmap::indexmap! {
                    "user_id".to_string() => user_id.into(),
                    "data".to_string() => data.clone().into(),
                },
                ectx.guild_id,
            )
            .await?;

            Ok(())
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
    uses_custom_template: bool,
) -> Result<bool, silverpelt::Error> {
    if event_name == "MESSAGE" {
        if !filters.contains(&event_name.to_string()) {
            return Ok(false);
        }

        if !uses_custom_template {
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
    event_data: indexmap::IndexMap<String, Field>,
    guild_id: serenity::model::id::GuildId,
) -> Result<(), silverpelt::Error> {
    let sinks = super::cache::get_sinks(guild_id, &data.pool).await?;

    if sinks.is_empty() {
        return Ok(());
    }

    for sink in sinks.iter() {
        // Verify event dispatch
        if !should_dispatch_event(
            event_name,
            {
                // False positive, unwrap_or_default cannot be used here as it moves the event out of the sink
                #[allow(clippy::manual_unwrap_or_default)]
                if let Some(ref events) = sink.events {
                    events
                } else {
                    &[]
                }
            },
            {
                if let Some(ref template) = sink.template {
                    !template.is_empty()
                } else {
                    false
                }
            },
        )
        .await?
        {
            continue;
        }

        let template = {
            if let Some(ref template) = sink.template {
                if !template.is_empty() {
                    templating::Template::Named(template.clone())
                } else {
                    // Load default template
                    templating::Template::Raw(load_embedded_event_template(event_name)?)
                }
            } else {
                templating::Template::Raw(load_embedded_event_template(event_name)?)
            }
        };

        let discord_reply = templating::execute::<_, Option<templating::core::messages::Message>>(
            guild_id,
            template,
            data.pool.clone(),
            botox::cache::CacheHttpImpl::from_ctx(ctx),
            data.reqwest.clone(),
            AuditLogContext {
                event_titlename: event_titlename.to_string(),
                event_name: event_name.to_string(),
                event_data: event_data.clone(),
            },
        )
        .await;

        let discord_reply = match discord_reply {
            Ok(reply) => {
                if let Some(reply) = reply {
                    match templating::core::messages::to_discord_reply(reply) {
                        Ok(reply) => reply,
                        Err(e) => {
                            let embed = serenity::all::CreateEmbed::default()
                                .description(format!("Failed to render template: {}", e));

                            templating::core::messages::DiscordReply {
                                embeds: vec![embed],
                                ..Default::default()
                            }
                        }
                    }
                } else {
                    continue;
                }
            }
            Err(e) => {
                let embed = serenity::all::CreateEmbed::default()
                    .description(format!("Failed to render template: {}", e));

                templating::core::messages::DiscordReply {
                    embeds: vec![embed],
                    ..Default::default()
                }
            }
        };

        let channel: ChannelId = sink.sink.parse()?;

        let mut message = CreateMessage::default().embeds(discord_reply.embeds);

        if let Some(content) = discord_reply.content {
            message = message.content(content);
        }

        match channel.send_message(&ctx.http, message).await {
            Ok(_) => {}
            Err(e) => {
                warn!(
                    "Failed to send audit log event to channel: {} [sink id: {}]",
                    e, sink.id
                );

                if let serenity::Error::Http(serenity::http::HttpError::UnsuccessfulRequest(
                    ref err,
                )) = e
                {
                    match err.status_code {
                        reqwest::StatusCode::FORBIDDEN
                        | reqwest::StatusCode::UNAUTHORIZED
                        | reqwest::StatusCode::NOT_FOUND
                        | reqwest::StatusCode::GONE => {
                            sqlx::query!(
                                "UPDATE auditlogs__sinks SET broken = true WHERE id = $1",
                                sink.id
                            )
                            .execute(&data.pool)
                            .await?;
                        }
                        _ => {}
                    }
                }
            }
        };
    }

    Ok(())
}

/// A AuditLogContext is a context for message templates
/// that can be accessed in audit log templates
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct AuditLogContext {
    pub event_titlename: String,
    pub event_name: String,
    pub event_data: indexmap::IndexMap<String, Field>,
}

#[typetag::serde]
impl templating::Context for AuditLogContext {}
