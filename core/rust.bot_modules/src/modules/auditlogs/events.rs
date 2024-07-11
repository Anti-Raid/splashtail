use crate::{silverpelt::EventHandlerContext, Data, Error};
use gwevent::field::CategorizedField;
use include_dir::{include_dir, Dir};
use log::warn;
use poise::serenity_prelude::FullEvent;
use serenity::all::{ChannelId, CreateMessage};
use std::sync::Arc;

static DEFAULT_TEMPLATES: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/src/modules/auditlogs/templates");

pub fn load_embedded_event_template(event: &str) -> Result<String, Error> {
    let template = match DEFAULT_TEMPLATES.get_file(format!("{}.tera", event)) {
        Some(template) => template,
        None => {
            // Load default.tera
            DEFAULT_TEMPLATES
                .get_file("default.tera")
                .ok_or("Failed to load default template")?
        }
    };

    let template_str = template.contents_utf8().ok_or("Failed to load template")?;

    Ok(template_str.to_string())
}

#[inline]
pub const fn not_audit_loggable_event() -> &'static [&'static str] {
    &[
        "CACHE_READY",         // Internal
        "INTERACTION_CREATE",  // Spams too much / is useless
        "MESSAGE",             // Spams too much / is useless
        "RATELIMIT",           // Internal
        "GUILD_CREATE",        // Internal
        "GUILD_MEMBERS_CHUNK", // Internal
    ]
}

pub async fn event_listener(ectx: &EventHandlerContext) -> Result<(), Error> {
    let ctx = &ectx.serenity_context;
    let event = &ectx.full_event;

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

    let Some(expanded_event) = gwevent::core::expand_event(event.clone()) else {
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
        event_name,
        &event_titlename,
        expanded_event,
        ectx.guild_id,
    )
    .await
}

pub async fn check_event_matches(event_name: &str, filters: Vec<String>) -> Result<bool, Error> {
    // If empty, always return Ok
    if filters.is_empty() {
        return Ok(true);
    }

    let mut regexes = Vec::new();

    for filter in filters.iter() {
        if filter.starts_with("R/") && filter.len() > 2 {
            regexes.push(&filter[2..]);
        }

        if event_name == filter {
            return Ok(true);
        }
    }

    for regex in regexes {
        match crate::silverpelt::silverpelt_cache::SILVERPELT_CACHE
            .regex_match(regex, event_name)
            .await
        {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(e) => {
                log::warn!("Failed to match regex: {}", e);
            }
        };
    }

    Ok(false)
}

pub async fn dispatch_audit_log(
    ctx: &serenity::client::Context,
    event_name: &str,
    event_titlename: &str,
    expanded_event: indexmap::IndexMap<String, CategorizedField>,
    guild_id: serenity::model::id::GuildId,
) -> Result<(), Error> {
    let mut event_embed: Option<serenity::all::CreateEmbed<'_>> = None;

    let user_data = ctx.data::<Data>();

    let sinks = sqlx::query!("SELECT id, type AS typ, sink, events, embed_template FROM auditlogs__sinks WHERE guild_id = $1 AND broken = false", guild_id.to_string())
        .fetch_all(&user_data.pool)
        .await?;

    if sinks.is_empty() {
        return Ok(());
    }

    for sink in sinks {
        // Verify event in whitelisted event list, if events is set
        if let Some(events) = sink.events {
            if !check_event_matches(event_name, events).await? {
                continue;
            }
        }

        let embed = if let Some(ref e) = event_embed {
            e.clone()
        } else {
            let mut tera = {
                if let Some(ref embed_template) = sink.embed_template {
                    templating::compile_template(
                        embed_template,
                        templating::CompileTemplateOptions {
                            ignore_cache: false,
                            cache_result: true,
                        },
                    )
                    .await?
                } else {
                    let template_str = load_embedded_event_template(event_name)?;

                    templating::compile_template(
                        &template_str,
                        templating::CompileTemplateOptions {
                            ignore_cache: false,
                            cache_result: true,
                        },
                    )
                    .await?
                }
            };

            // Add gwevent templater
            tera.register_filter(
                "formatter__gwevent_field",
                gwevent::templating::FieldFormatter {
                    is_categorized_default: true,
                },
            );

            let mut ctx = templating::make_templating_context();
            ctx.insert("event_name", event_name)?;
            ctx.insert("event_titlename", event_titlename)?;
            ctx.insert("event", &expanded_event)?;

            let templated = templating::execute_template(&mut tera, Arc::new(ctx)).await?;
            let e = templating::to_embed(templated)?;

            event_embed = Some(e.clone());

            e
        };

        match sink.typ.as_str() {
            "channel" => {
                let channel: ChannelId = sink.sink.parse()?;

                match channel
                    .send_message(&ctx.http, CreateMessage::default().embed(embed.clone()))
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        warn!(
                            "Failed to send audit log event to channel: {} [sink id: {}]",
                            e, sink.id
                        );

                        if let serenity::Error::Http(
                            serenity::http::HttpError::UnsuccessfulRequest(ref err),
                        ) = e
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
                                    .execute(&user_data.pool)
                                    .await?;
                                }
                                _ => {}
                            }
                        }
                    }
                };
            }
            "discord_webhook" => {
                let parsed_sink = sink.sink.parse()?;
                let Some((id, token)) = serenity::utils::parse_webhook(&parsed_sink) else {
                    warn!(
                        "Failed to parse webhook URL: {} [sink id: {}]",
                        sink.sink, sink.id
                    );
                    continue;
                };

                let webhook_proxyurl = format!(
                    "{base_url}/api/v10/webhooks/{id}/{token}",
                    base_url = config::CONFIG.meta.proxy.get(),
                    id = id,
                    token = token
                );

                let req = match user_data
                    .reqwest
                    .post(&webhook_proxyurl)
                    .json(&serde_json::json!({
                        "embeds": [embed.clone()]
                    }))
                    .header("Content-Type", "application/json")
                    .header(
                        "User-Agent",
                        "DiscordBot/0.1 (Anti-Raid, https://github.com/anti-raid)",
                    )
                    .send()
                    .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        warn!(
                            "Failed to send audit log event to webhook: {} [sink id: {}]",
                            e, sink.id
                        );
                        continue;
                    }
                };

                let status = req.status();
                // reqwest::StatusCode::FORBIDDEN | reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::NOT_FOUND | reqwest::StatusCode::GONE
                if status == reqwest::StatusCode::FORBIDDEN
                    || status == reqwest::StatusCode::UNAUTHORIZED
                    || status == reqwest::StatusCode::NOT_FOUND
                    || status == reqwest::StatusCode::GONE
                {
                    let text = req.text().await?;
                    warn!(
                        "Failed to send audit log event to webhook ({} [broken]): {} [sink id: {}]",
                        status, text, sink.id
                    );

                    sqlx::query!(
                        "UPDATE auditlogs__sinks SET broken = true WHERE id = $1",
                        sink.id
                    )
                    .execute(&user_data.pool)
                    .await?;
                } else if !status.is_success() {
                    let text = req.text().await?;
                    warn!(
                        "Failed to send audit log event to webhook ({}): {} [sink id: {}]",
                        status, text, sink.id
                    );
                }
            }
            _ => {
                warn!("Unknown sink type: {} [sink id: {}]", sink.typ, sink.id);
            }
        }
    }

    Ok(())
}
