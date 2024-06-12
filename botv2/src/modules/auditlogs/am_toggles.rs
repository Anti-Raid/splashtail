use crate::silverpelt::value::Value;
use futures::future::FutureExt;

pub async fn setup(_data: &crate::Data) -> Result<(), crate::Error> {
    crate::ipc::animus_magic::bot::dynamic::PERMODULE_FUNCTIONS.insert(
        ("auditlogs".to_string(), "check_all_events".to_string()),
        Box::new(move |_, options| check_all_events(options).boxed()),
    );

    crate::ipc::animus_magic::bot::dynamic::PERMODULE_FUNCTIONS.insert(
        ("auditlogs".to_string(), "check_channel".to_string()),
        Box::new(move |cache_http, options| check_channel(cache_http, options).boxed()),
    );

    Ok(())
}

/// Arguments:
///
/// - events: Vec<String>
pub async fn check_all_events(
    value: &indexmap::IndexMap<String, Value>,
) -> Result<(), crate::Error> {
    let events = match value.get("events") {
        Some(Value::List(a)) => a,
        Some(Value::None) => return Ok(()),
        _ => return Err("`events` could not be parsed".into()),
    };

    // Parse each array element as a string
    let events: Vec<String> = events
        .iter()
        .map(|v| match v {
            Value::String(s) => Ok(s.clone()),
            _ => Err("`events` could not be parsed".into()),
        })
        .collect::<Result<Vec<String>, crate::Error>>()?;

    super::checks::check_all_events(events).await?;

    Ok(())
}

/// Arguments:
///
/// - channel_id: serenity::model::id::ChannelId
/// - guild_id: serenity::model::id::GuildId
pub async fn check_channel(
    cache_http: &crate::CacheHttpImpl,
    value: &indexmap::IndexMap<String, Value>,
) -> Result<(), crate::Error> {
    let channel_id = match value.get("channel_id") {
        Some(Value::String(s)) => s.parse::<serenity::all::ChannelId>()?,
        _ => return Err("`channel_id` could not be parsed".into()),
    };

    let guild_id = match value.get("guild_id") {
        Some(Value::String(s)) => s.parse::<serenity::all::GuildId>()?,
        _ => return Err("`guild_id` could not be parsed".into()),
    };

    super::checks::check_channel(cache_http, channel_id, guild_id).await?;

    Ok(())
}
