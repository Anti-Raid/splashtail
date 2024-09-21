use super::Job;
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
        Arc<Job>,
    ) -> Pin<Box<dyn Future<Output = Result<(), crate::Error>> + Send>>,
    to: PollTaskOptions,
) -> Result<(), crate::Error> {
    let interval = to.interval;
    let timeout_nostatuschange = to.timeout_nostatuschange;
    let duration = std::time::Duration::from_secs(interval);
    let mut interval = tokio::time::interval(duration);
    let id = sqlx::types::uuid::Uuid::parse_str(id)?;
    let mut prev_job: Option<Arc<Job>> = None;

    let mut last_statuschange = tokio::time::Instant::now();
    loop {
        interval.tick().await;

        if timeout_nostatuschange > 0
            && tokio::time::Instant::now() - last_statuschange
                > tokio::time::Duration::from_secs(timeout_nostatuschange)
        {
            return Err(format!(
                "Job poll timeout of {} seconds reached without status change",
                timeout_nostatuschange
            )
            .into());
        }

        let job = Arc::new(super::Job::from_id(id, pool).await?);

        if let Some(ref prev_job) = prev_job {
            if prev_job.state == job.state && job.statuses == prev_job.statuses {
                continue;
            }
        }

        prev_job = Some(job.clone());

        last_statuschange = tokio::time::Instant::now();

        func(cache_http, job.clone()).await?;

        if job.state != "pending" && job.state != "running" {
            break;
        }
    }

    drop(prev_job); // Drop prev_task

    Ok(())
}
