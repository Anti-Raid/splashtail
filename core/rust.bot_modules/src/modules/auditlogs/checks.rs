pub async fn check_all_events(events: Vec<String>) -> Result<(), crate::Error> {
    let is_killed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    let res_killed = is_killed.clone();
    let res = tokio::time::timeout(
        std::time::Duration::from_millis(250),
        tokio::task::spawn_blocking(move || {
            let supported_events = gwevent::core::event_list();
            let not_audit_loggable = super::events::not_audit_loggable_event();

            for event in events {
                if res_killed.load(std::sync::atomic::Ordering::SeqCst) {
                    return Err("Killed".to_string());
                }

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

                if not_audit_loggable.contains(&event.as_str()) {
                    return Err(format!(
                        "Event `{}` is explicitly not audit loggable yet!",
                        trimmed,
                    ));
                }
            }

            Ok(())
        }),
    )
    .await??;

    is_killed.store(true, std::sync::atomic::Ordering::SeqCst); // Kill the task when possible

    res.map_err(|e| e.into())
}
