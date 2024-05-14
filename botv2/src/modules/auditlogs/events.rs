use crate::{silverpelt::EventHandlerContext, Data, Error};
use gwevent::field_type::FieldType;
use log::warn;
use poise::serenity_prelude::FullEvent;
use serenity::all::{ChannelId, CreateMessage, Mentionable};

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

fn resolve_gwevent_field(field: &FieldType) -> Result<String, crate::Error> {
    // Given a serde_json::Value, loop over all keys and resolve them (recursively if needed)
    fn serde_resolver(v: &serde_json::Value) -> Result<String, crate::Error> {
        match v {
            serde_json::Value::Null => Ok("None".to_string()),
            serde_json::Value::Bool(b) => Ok(if *b { "Yes" } else { "No" }.to_string()),
            serde_json::Value::Number(n) => Ok(n.to_string()),
            serde_json::Value::String(s) => Ok(s.to_string()),
            serde_json::Value::Object(o) => {
                let mut resolved = Vec::new();

                for (k, v) in o.iter() {
                    resolved.push(format!(
                        "{} => {}",
                        k.split('_')
                            .map(|s| {
                                let mut c = s.chars();
                                match c.next() {
                                    None => String::new(),
                                    Some(f) => f.to_uppercase().chain(c).collect(),
                                }
                            })
                            .collect::<Vec<String>>()
                            .join(" "),
                        serde_resolver(v)?
                    ));
                }

                Ok(resolved.join("\n"))
            }
            serde_json::Value::Array(v) => {
                let mut resolved = Vec::new();

                for i in v.iter() {
                    resolved.push(serde_resolver(i)?);
                }

                Ok(resolved.join("\n\n"))
            }
        }
    }

    match field {
        FieldType::Strings(s) => {
            let joined = s.join(", ");
            Ok(joined)
        }
        FieldType::Bool(b) => Ok(if *b { "Yes" } else { "No" }.to_string()),
        FieldType::Number(n) => Ok(n.to_string()),
        FieldType::Permissions(p) => {
            let mut perms = Vec::new();

            for ip in p.iter() {
                perms.push(format!("{} ({})", ip, ip.bits()));
            }

            Ok(perms.join(", "))
        }
        FieldType::PermissionOverwrites(p) => {
            let mut perms = Vec::new();

            for ip in p.iter() {
                perms.push(format!("Allow={}, Deny={}", ip.allow, ip.deny));
            }

            Ok(perms.join(", "))
        }
        FieldType::GuildMemberFlags(p) => {
            let p_vec = p
                .iter()
                .map(|x| format!("{:#?}", x))
                .collect::<Vec<String>>();

            if p_vec.is_empty() {
                return Ok("None".to_string());
            }

            Ok(p_vec.join(", "))
        }
        FieldType::UserIds(u) => {
            let mut users = Vec::new();

            for iu in u.iter() {
                users.push(iu.mention().to_string());
            }

            Ok(users.join(", "))
        }
        FieldType::Channels(c) => {
            let mut channels = Vec::new();

            for ic in c.iter() {
                channels.push(ic.mention().to_string());
            }

            Ok(channels.join(", "))
        }
        FieldType::NsfwLevels(n) => {
            let mut nsfw_levels = Vec::new();

            for inl in n.iter() {
                nsfw_levels.push(format!("{:#?}", inl));
            }

            Ok(nsfw_levels.join(", "))
        }
        FieldType::Roles(r) => {
            let mut roles = Vec::new();

            for ir in r.iter() {
                roles.push(ir.mention().to_string());
            }

            Ok(roles.join(", "))
        }
        FieldType::GenericIds(g) => {
            let mut generic_ids = Vec::new();

            for ig in g.iter() {
                generic_ids.push(ig.to_string());
            }

            Ok(generic_ids.join(", "))
        }
        FieldType::Timestamp(t) => Ok(t.to_string()),
        FieldType::Attachment(a) => Ok(a.url.to_string()),
        FieldType::JsonValue(v) => match serde_json::to_string(v) {
            Ok(s) => Ok(format!("``{}``", s)),
            Err(e) => Err(e.into()),
        },
        FieldType::None => Ok("None".to_string()),
        _ => {
            let s = serde_resolver(&serde_json::to_value(field)?)?;
            Ok(s)
        }
    }
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

fn create_audit_log_embed<'a>(
    event_titlename: &'a str,
    expanded_event: &'a indexmap::IndexMap<(String, String), FieldType>,
) -> Result<serenity::all::CreateEmbed<'a>, crate::Error> {
    let mut event_embed_len = event_titlename.len();
    let mut event_embed = serenity::all::CreateEmbed::new().title(event_titlename);

    let mut compiled_fields: indexmap::IndexMap<String, indexmap::IndexMap<String, String>> =
        indexmap::IndexMap::new();

    // Keep adding fields until length becomes > 6000
    // TODO: Improve embed display
    for ((category, k), v) in expanded_event {
        let kc = k
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

        let resolved_field = resolve_gwevent_field(v)?;

        let mut value = resolved_field.trim();

        if value.is_empty() {
            continue;
        }

        // TODO: Support/handle embed limits better
        if value.len() > 490 {
            value = &value[..490];
        }

        let mut field_len = kc.len() + value.len();

        if field_len > 1024 {
            value = &value[..1024 - kc.len()];
            field_len = 1024;
        }

        if event_embed_len + field_len > 6000 {
            break;
        }

        event_embed_len += field_len;

        // Use the indexmap entry api to insert the field
        let entry = compiled_fields.entry(category.clone()).or_default();
        entry.insert(kc, value.to_string());
    }

    // Now create the embed
    for (category, fields) in compiled_fields {
        let mut category_str = String::new();

        for (k, v) in fields {
            let v = format!("**{}:** {}\n", k, v);
            category_str.push_str(&v);
        }

        event_embed = event_embed.field(category, category_str, false);
    }

    Ok(event_embed)
}

pub async fn dispatch_audit_log(
    ctx: &serenity::client::Context,
    event_name: &str,
    event_titlename: &str,
    expanded_event: indexmap::IndexMap<(String, String), FieldType>,
    guild_id: serenity::model::id::GuildId,
) -> Result<(), Error> {
    let mut event_embed: Option<serenity::all::CreateEmbed<'_>> = None;

    let user_data = ctx.data::<Data>();

    let sinks = sqlx::query!("SELECT id, type AS typ, sink, events FROM auditlogs__sinks WHERE guild_id = $1 AND broken = false", guild_id.to_string())
        .fetch_all(&user_data.pool)
        .await?;

    for sink in sinks {
        // Verify event in whitelisted event list, if events is set
        if let Some(events) = sink.events {
            if !check_event_matches(event_name, events).await? {
                continue;
            }
        }

        match sink.typ.as_str() {
            "channel" => {
                let embed = if let Some(ref e) = event_embed {
                    e.clone()
                } else {
                    let e = create_audit_log_embed(event_titlename, &expanded_event)?;
                    event_embed = Some(e.clone());

                    e
                };

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
                let embed = if let Some(ref e) = event_embed {
                    e.clone()
                } else {
                    let e = create_audit_log_embed(event_titlename, &expanded_event)?;

                    event_embed = Some(e.clone());
                    e
                };

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
                    base_url = crate::config::CONFIG.meta.proxy.get(),
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
