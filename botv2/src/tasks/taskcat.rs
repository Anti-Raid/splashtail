use log::{error, info};
use once_cell::sync::Lazy;
use std::time::Duration;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use tokio::sync::Mutex;
use tokio::task::JoinSet;

static TASK_MUTEX: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));

#[derive(EnumIter, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Task {
    UpdateStatus,
}

impl Task {
    /// Whether or not the task is enabled
    pub fn enabled(&self) -> bool {
        match self {
            Task::UpdateStatus => true,
        }
    }

    /// How often the task should run
    pub fn duration(&self) -> Duration {
        match self {
            Task::UpdateStatus => Duration::from_secs(300),
        }
    }

    /// Description of the task
    pub fn description(&self) -> &'static str {
        match self {
            Task::UpdateStatus => "Updating statuses",
        }
    }

    /// Function to run the task
    pub async fn run(
        &self,
        pool: &sqlx::PgPool,
        cache_http: &bothelpers::cache::CacheHttpImpl,
        ctx: &serenity::client::Context,
    ) -> Result<(), crate::Error> {
        match self {
            Task::UpdateStatus => {
                crate::tasks::update_status::update_status(pool, cache_http, ctx).await
            }
        }
    }
}

/// Function to start all tasks
pub async fn start_all_tasks(
    pool: sqlx::PgPool,
    cache_http: bothelpers::cache::CacheHttpImpl,
    ctx: serenity::client::Context,
) -> ! {
    // Start tasks
    let mut set = JoinSet::new();

    for task in Task::iter() {
        if !task.enabled() {
            continue;
        }

        set.spawn(crate::tasks::taskcat::taskcat(
            pool.clone(),
            cache_http.clone(),
            ctx.clone(),
            task,
        ));
    }

    if let Some(res) = set.join_next().await {
        if let Err(e) = res {
            error!("Error while running task: {}", e);
        }

        info!("Task finished when it shouldn't have");
        std::process::abort();
    }

    info!("All tasks finished when they shouldn't have");
    std::process::abort();
}

/// Function that manages a task
async fn taskcat(
    pool: sqlx::PgPool,
    cache_http: bothelpers::cache::CacheHttpImpl,
    ctx: serenity::client::Context,
    task: Task,
) -> ! {
    let duration = task.duration();
    let description = task.description();

    // Ensure multiple tx's are not created at the same time
    tokio::time::sleep(duration).await;

    let mut interval = tokio::time::interval(duration);

    loop {
        interval.tick().await;

        let guard = TASK_MUTEX.lock().await;

        log::info!(
            "TASK: {} ({}s interval) [{}]",
            task.to_string(),
            duration.as_secs(),
            description
        );

        if let Err(e) = task.run(&pool, &cache_http, &ctx).await {
            log::error!("TASK {} ERROR'd: {:?}", task.to_string(), e);
        }

        drop(guard);
    }
}
