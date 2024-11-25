use crate::Job;
use futures_util::Stream;
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

pub fn reactive(
    pool: &sqlx::PgPool,
    id: &str,
    to: PollTaskOptions,
) -> Result<impl Stream<Item = Result<Option<Arc<Job>>, splashcore_rs::Error>>, splashcore_rs::Error>
{
    let interval = to.interval;
    let timeout_nostatuschange = to.timeout_nostatuschange;
    let duration = std::time::Duration::from_secs(interval);
    let interval = tokio::time::interval(duration);
    let id = sqlx::types::uuid::Uuid::parse_str(id)?;
    let last_statuschange = tokio::time::Instant::now();

    Ok(futures_util::stream::unfold(
        JobserverStreamState {
            pool: pool.clone(),
            id,
            timeout_nostatuschange,
            prev_job: None,
            interval,
            last_statuschange,
            at_end: false,
        },
        |state| async move {
            let mut state = state;

            if let Some(ref prev_job) = state.prev_job {
                if prev_job.state == "completed" {
                    if state.at_end {
                        return None;
                    } else {
                        state.at_end = true;
                    }
                } else {
                    state.at_end = false;
                }
            }

            state.interval.tick().await;

            if state.timeout_nostatuschange > 0
                && tokio::time::Instant::now() - state.last_statuschange
                    > tokio::time::Duration::from_secs(state.timeout_nostatuschange)
            {
                return Some((
                    Err(format!(
                        "Job poll timeout of {} seconds reached without status change",
                        state.timeout_nostatuschange
                    )
                    .into()),
                    state,
                ));
            }

            let job = match super::Job::from_id(state.id, &state.pool).await {
                Ok(job) => Arc::new(job),
                Err(e) => return Some((Err(e), state)),
            };

            if let Some(ref prev_job) = state.prev_job {
                if prev_job.state == job.state && job.statuses == prev_job.statuses {
                    return Some((Ok(None), state));
                }
            }

            state.prev_job = Some(job.clone());
            state.last_statuschange = tokio::time::Instant::now();

            return Some((Ok(Some(job.clone())), state));
        },
    ))
}

pub struct JobserverStreamState {
    pool: sqlx::PgPool,
    id: sqlx::types::Uuid,
    timeout_nostatuschange: u64,
    prev_job: Option<Arc<Job>>,
    interval: tokio::time::Interval,
    last_statuschange: tokio::time::Instant,
    at_end: bool,
}
