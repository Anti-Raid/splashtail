pub async fn check_all_events(events: Vec<String>) -> Result<(), crate::Error> {
    let res = tokio::time::timeout(
        std::time::Duration::from_millis(1000),
        tokio::task::spawn_blocking(move || {
            let supported_events = gwevent::core::event_list();

            for event in events {
                let trimmed = event.trim().to_string();

                if trimmed.is_empty() {
                    continue;
                }

                // All Anti-Raid events are filterable
                if trimmed.starts_with("AR/") {
                    continue;
                }

                // Regex compile check
                if trimmed.starts_with("R/") {
                    if let Err(e) = regex::Regex::new(&trimmed) {
                        return Err(format!(
                            "Event `{}` is not a valid regex. Error: {}",
                            trimmed, e
                        ));
                    }
                }

                let event = trimmed.to_uppercase();

                if !supported_events.contains(&event.as_str()) {
                    return Err(format!(
                        "Event `{}` is not a valid event. Please pick one of the following: {}",
                        trimmed,
                        supported_events.join(", ")
                    ));
                }
            }

            Ok(())
        }),
    )
    .await??;

    res.map_err(|e| e.into())
}
