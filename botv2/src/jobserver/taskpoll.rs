use crate::{impls::cache::CacheHttpImpl, jobserver::Task};
use std::pin::Pin;
use std::sync::Arc;
use std::future::Future;
use serde_json::Value;

pub struct PollTaskOptions {
    /// The interval at which to update/poll at in seconds
    pub interval: Option<u64>,
}

fn _to_string(v: &Option<&Value>) -> String {
    let v = match v {
        Some(v) => v,
        None => return "null".to_string(),
    };

    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(a) => a.iter().map(|v| _to_string(&Some(v))).collect::<Vec<_>>().join(", "),
        Value::Object(o) => o.iter().map(|(k, v)| format!("{}={}", k, _to_string(&Some(v)))).collect::<Vec<_>>().join(", "),
    }
}

pub fn embed<'a>(task: &Task) -> Result<poise::CreateReply<'a>, crate::Error> {
    let mut task_statuses: Vec<String> = Vec::new();
    let mut task_statuses_length = 0;
    let mut components = Vec::new();

    let task_state = &task.state;

    for status in &task.statuses {
        if task_statuses_length > 2500 {
            // Keep removing elements from start of array until we are under 2500 characters
            while task_statuses_length > 2500 {
                let removed = task_statuses.remove(0);
                task_statuses_length -= removed.len();
            }
        }

        let mut add = format!("`{}` {}", status.level, status.msg);

        let mut vs = Vec::new();

        let bdi = status.bot_display_ignore.clone().unwrap_or_default();

        for (k, v) in status.extra_info.iter() {
            if bdi.contains(k) {
                continue;
            }

            vs.push(format!("{}={}", k, serde_json::to_string(v)?));
        }

        if !vs.is_empty() {
            add += &format!(" {}", vs.join(", "));
        }

        add = add.chars().take(500).collect::<String>() + if add.len() > 500 { "..." } else { "" };

        add += &format!(" | <t:{}:R>", status.ts.round());

        task_statuses_length += if add.len() > 500 { 500 } else { add.len() };
        task_statuses.push(add);
    }

    let mut description = format!(
        "{} Task state: {}\nTask ID: {}\n\n{}",
        match task_state.as_str() {
            "pending" => ":hourglass:",
            "running" => ":hourglass_flowing_sand:",
            "completed" => ":white_check_mark:",
            "failed" => ":x:",
            _ => ":question:",
        },
        task_state,
        task.task_id,
        task_statuses.join("\n")
    );

    if task.state == "completed" {
        if let Some(ref output) = task.output {
            let furl = format!("{}/tasks/{}/ioauth/download-link", crate::config::CONFIG.sites.api.get(), task.task_id);
            description += &format!("\n\n:link: [Download {}]({})", output.filename, &furl);

            components.push(
                poise::serenity_prelude::CreateActionRow::Buttons(
                    vec![
                        poise::serenity_prelude::CreateButton::new_link(
                            furl,
                        )
                        .label("Download")
                        .emoji('📥'),
                    ]
                ),
            );
        }
    }

    let embed = poise::serenity_prelude::CreateEmbed::default()
        .title("Task Status")
        .description(description)
        .color(poise::serenity_prelude::Colour::DARK_GREEN);

    let msg = poise::CreateReply::default().embed(embed).components(components);

    Ok(msg)
}

pub async fn reactive(
    cache_http: &CacheHttpImpl,
    pool: &sqlx::PgPool,
    task_id: &str,
    mut func: impl FnMut(&CacheHttpImpl, Arc<Task>) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send>>,
    to: PollTaskOptions,
) -> Result<(), crate::Error> {
    let interval = to.interval.unwrap_or(1);
    let duration = std::time::Duration::from_secs(interval);
    let mut interval = tokio::time::interval(duration);
    let task_id = sqlx::types::uuid::Uuid::parse_str(task_id)?;
    let mut prev_task: Option<Arc<Task>> = None;
    loop {
        interval.tick().await;

        let task = Arc::new(super::Task::from_id(task_id, pool).await?);

        if let Some(ref prev_task) = prev_task {
            if prev_task.state == task.state && task.statuses == prev_task.statuses {
                continue;
            }
        }

        prev_task = Some(task.clone());

        func(cache_http, task.clone()).await?;

        if task.state != "pending" && task.state != "running" {
            break;
        }
    }

    drop(prev_task); // Drop prev_task

    Ok(())
}