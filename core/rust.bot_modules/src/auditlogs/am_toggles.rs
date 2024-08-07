use futures_util::future::FutureExt;
use splashcore_rs::value::Value;

pub async fn setup(data: &base_data::Data) -> Result<(), base_data::Error> {
    data.props.add_permodule_function(
        "auditlogs",
        "check_all_events",
        Box::new(move |_, options| check_all_events(options).boxed()),
    );

    Ok(())
}

/// Arguments:
///
/// - events: Vec<String>
pub async fn check_all_events(
    value: &indexmap::IndexMap<String, Value>,
) -> Result<(), base_data::Error> {
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
        .collect::<Result<Vec<String>, base_data::Error>>()?;

    super::checks::check_all_events(events).await?;

    Ok(())
}
