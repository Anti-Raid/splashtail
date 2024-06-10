use futures::future::FutureExt;
use crate::silverpelt::value::Value;

pub async fn setup(_data: &crate::Data) -> Result<(), crate::Error> {
    crate::ipc::animus_magic::bot::dynamic::PERMODULE_FUNCTIONS.insert(
        ("auditlogs".to_string(), "check_all_events".to_string()),
        Box::new(move |_, options| check_all_events(options).boxed()),
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

    super::core::check_all_events(events).await?;

    Ok(())
}
