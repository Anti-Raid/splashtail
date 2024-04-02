use log::{error, info};
use once_cell::sync::Lazy;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use futures::future::BoxFuture;

static TASK_MUTEX: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));

pub type RunFunction = Box<
dyn Send
+ Sync
+ for<'a> Fn(
    &'a sqlx::PgPool,
    &'a bothelpers::cache::CacheHttpImpl,
    &'a serenity::all::Context,
) -> BoxFuture<'a, Result<(), crate::Error>>,
>;

pub struct Task {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub duration: Duration,
    pub run: RunFunction
}

/// Function to start all tasks
pub async fn start_all_tasks(
    tasks: Vec<Task>,
    pool: sqlx::PgPool,
    cache_http: bothelpers::cache::CacheHttpImpl,
    ctx: serenity::client::Context,
) -> ! {
    // Start tasks
    let mut set = JoinSet::new();

    for task in tasks {
        if !task.enabled {
            continue;
        }

        info!("Starting task: {}", task.name);

        set.spawn(taskcat(
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
    // Ensure multiple tx's are not created at the same time
    tokio::time::sleep(task.duration).await;

    let mut interval = tokio::time::interval(task.duration);

    loop {
        interval.tick().await;

        let guard = TASK_MUTEX.lock().await;

        log::info!(
            "TASK: {} ({}s interval) [{}]",
            task.name,
            task.duration.as_secs(),
            task.description
        );

        if let Err(e) = (task.run)(&pool, &cache_http, &ctx).await {
            log::error!("TASK {} ERROR'd: {:?}", task.name, e);
        }

        drop(guard);
    }
}
