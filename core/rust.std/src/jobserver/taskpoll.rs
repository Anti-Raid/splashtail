use super::Task;
use botox::cache::CacheHttpImpl;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct PollTaskOptions {
    /// The interval at which to update/poll at in seconds
    pub interval: u64,

    /// The timeout in seconds to wait for the task to change in status
    pub timeout_nostatuschange: u64,
}

impl Default for PollTaskOptions {
    fn default() -> Self {
        PollTaskOptions {
            interval: 1,
            timeout_nostatuschange: 300,
        }
    }
}

pub async fn reactive(
    cache_http: &CacheHttpImpl,
    pool: &sqlx::PgPool,
    id: &str,
    mut func: impl FnMut(
        &CacheHttpImpl,
        Arc<Task>,
    ) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send>>,
    to: PollTaskOptions,
) -> Result<(), crate::Error> {
    let interval = to.interval;
    let timeout_nostatuschange = to.timeout_nostatuschange;
    let duration = std::time::Duration::from_secs(interval);
    let mut interval = tokio::time::interval(duration);
    let id = sqlx::types::uuid::Uuid::parse_str(id)?;
    let mut prev_task: Option<Arc<Task>> = None;

    let mut last_statuschange = tokio::time::Instant::now();
    loop {
        interval.tick().await;

        if timeout_nostatuschange > 0
            && tokio::time::Instant::now() - last_statuschange
                > tokio::time::Duration::from_secs(timeout_nostatuschange)
        {
            return Err(format!(
                "Task status timeout of {} seconds reached",
                timeout_nostatuschange
            )
            .into());
        }

        let task = Arc::new(super::Task::from_id(id, pool).await?);

        if let Some(ref prev_task) = prev_task {
            if prev_task.state == task.state && task.statuses == prev_task.statuses {
                continue;
            }
        }

        prev_task = Some(task.clone());

        last_statuschange = tokio::time::Instant::now();

        func(cache_http, task.clone()).await?;

        if task.state != "pending" && task.state != "running" {
            break;
        }
    }

    drop(prev_task); // Drop prev_task

    Ok(())
}
