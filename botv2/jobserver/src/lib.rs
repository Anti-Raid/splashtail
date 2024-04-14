pub mod taskpoll;

use splashcore_rs::objectstore::ObjectStore;
use bothelpers::utils::get_icon_of_state;
use indexmap::IndexMap;
use object_store::path::Path;
use object_store::signer::Signer;
use reqwest::Method;
use sqlx::{types::uuid::Uuid, PgPool};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Rust internal/special type to better serialize/speed up task embed creation
#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct TaskStatuses {
    pub level: String,
    pub msg: String,
    pub ts: f64,
    #[serde(rename = "botDisplayIgnore")]
    pub bot_display_ignore: Option<Vec<String>>,

    #[serde(flatten)]
    pub extra_info: IndexMap<String, serde_json::Value>,
}

pub struct Task {
    pub task_id: Uuid,
    pub task_name: String,
    pub output: Option<TaskOutput>,
    pub task_fields: IndexMap<String, serde_json::Value>,
    pub statuses: Vec<TaskStatuses>,
    pub task_for: TaskFor,
    pub expiry: Option<chrono::Duration>,
    pub state: String,
    pub resumable: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct TaskFor {
    pub id: String,
    pub target_type: String,
}

impl FromStr for TaskFor {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(2, '/');
        let target_type = split.next().ok_or("Invalid task for")?;
        let id = split.next().ok_or("Invalid task for")?;

        Ok(Self {
            id: id.to_string(),
            target_type: target_type.to_string(),
        })
    }
}

impl From<String> for TaskFor {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct TaskOutput {
    pub filename: String,
    pub segregated: bool,
}

/// TaskCreateResponse is the response upon creating a task
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct TaskCreateResponse {
    /// The ID of the newly created task
    pub task_id: String,
}

/// WrappedTaskCreateResponse is the response upon creating a task
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct WrappedTaskCreateResponse {
    pub tcr: TaskCreateResponse,
}

impl Task {
    /// Fetches a task from the database based on id
    pub async fn from_id(task_id: Uuid, pool: &PgPool) -> Result<Self, crate::Error> {
        let rec = sqlx::query!(
            "SELECT task_id, task_name, output, statuses, task_for, expiry, state, created_at, task_fields, resumable FROM tasks WHERE task_id = $1 ORDER BY created_at DESC",
            task_id,
        )
        .fetch_one(pool)
        .await?;

        let mut statuses = Vec::new();

        for status in &rec.statuses {
            let status = serde_json::from_value::<TaskStatuses>(status.clone())?;
            statuses.push(status);
        }

        let task = Task {
            task_id: rec.task_id,
            task_name: rec.task_name,
            output: rec
                .output
                .map(serde_json::from_value::<TaskOutput>)
                .transpose()?,
            task_fields: serde_json::from_value::<IndexMap<String, serde_json::Value>>(rec.task_fields)?,
            statuses,
            task_for: rec.task_for.into(),
            expiry: {
                if let Some(expiry) = rec.expiry {
                    let t = expiry.microseconds
                        + 60 * 1_000_000
                        + (expiry.days as i64) * 24 * 60 * 60 * 1_000_000
                        + (expiry.months as i64) * 30 * 24 * 60 * 60 * 1_000_000;
                    Some(chrono::Duration::microseconds(t))
                } else {
                    None
                }
            },
            state: rec.state,
            created_at: rec.created_at,
            resumable: rec.resumable,
        };

        Ok(task)
    }

    /// Fetches all tasks of a guild given guild id
    #[allow(dead_code)] // Will be used in the near future
    pub async fn from_guild(
        guild_id: serenity::all::GuildId,
        pool: &sqlx::PgPool,
    ) -> Result<Vec<Self>, crate::Error> {
        let recs = sqlx::query!(
            "SELECT task_id, task_name, output, statuses, task_for, expiry, state, created_at, task_fields, resumable FROM tasks WHERE task_for = $1",
            format!("g/{}", guild_id)
        )
        .fetch_all(pool)
        .await?;

        let mut tasks = Vec::new();

        for rec in recs {
            let mut statuses = Vec::new();

            for status in &rec.statuses {
                let status = serde_json::from_value::<TaskStatuses>(status.clone())?;
                statuses.push(status);
            }

            let task = Task {
                task_id: rec.task_id,
                task_name: rec.task_name,
                output: rec
                    .output
                    .map(serde_json::from_value::<TaskOutput>)
                    .transpose()?,
                task_fields: serde_json::from_value::<IndexMap<String, serde_json::Value>>(rec.task_fields)?,
                statuses,
                task_for: rec.task_for.into(),
                expiry: {
                    if let Some(expiry) = rec.expiry {
                        let t = expiry.microseconds
                            + 60 * 1_000_000
                            + (expiry.days as i64) * 24 * 60 * 60 * 1_000_000
                            + (expiry.months as i64) * 30 * 24 * 60 * 60 * 1_000_000;
                        Some(chrono::Duration::microseconds(t))
                    } else {
                        None
                    }
                },
                state: rec.state,
                created_at: rec.created_at,
                resumable: rec.resumable,
            };

            tasks.push(task);
        }

        Ok(tasks)
    }

    /// Returns all tasks with a specific guild ID and a specific task name
    pub async fn from_guild_and_task_name(
        guild_id: serenity::all::GuildId,
        task_name: &str,
        pool: &sqlx::PgPool,
    ) -> Result<Vec<Self>, crate::Error> {
        let recs = sqlx::query!(
            "SELECT task_id, task_name, output, statuses, task_for, expiry, state, created_at, task_fields, resumable FROM tasks WHERE task_for = $1 AND task_name = $2",
            format!("g/{}", guild_id),
            task_name,
        )
        .fetch_all(pool)
        .await?;

        let mut tasks = Vec::new();

        for rec in recs {
            let mut statuses = Vec::new();

            for status in &rec.statuses {
                let status = serde_json::from_value::<TaskStatuses>(status.clone())?;
                statuses.push(status);
            }

            let task = Task {
                task_id: rec.task_id,
                task_name: rec.task_name,
                output: rec
                    .output
                    .map(serde_json::from_value::<TaskOutput>)
                    .transpose()?,
                task_fields: serde_json::from_value::<IndexMap<String, serde_json::Value>>(rec.task_fields)?,
                statuses,
                task_for: rec.task_for.into(),
                expiry: {
                    if let Some(expiry) = rec.expiry {
                        let t = expiry.microseconds
                            + 60 * 1_000_000
                            + (expiry.days as i64) * 24 * 60 * 60 * 1_000_000
                            + (expiry.months as i64) * 30 * 24 * 60 * 60 * 1_000_000;
                        Some(chrono::Duration::microseconds(t))
                    } else {
                        None
                    }
                },
                state: rec.state,
                created_at: rec.created_at,
                resumable: rec.resumable,
            };

            tasks.push(task);
        }

        Ok(tasks)
    }

    pub fn format_task_for_simplex(&self) -> String {
        format!("{}/{}", self.task_for.target_type.to_lowercase(), self.task_for.id)
    }

    pub fn get_path(&self) -> Option<String> {
        if let Some(output) = &self.output {
            if output.segregated {
                Some(format!(
                    "{}/{}/{}",
                    self.format_task_for_simplex(),
                    self.task_name,
                    self.task_id,
                ))
            } else {
                Some(format!("tasks/{}", self.task_id))
            }
        } else {
            None
        }
    }

    pub fn get_file_path(&self) -> Option<String> {
        let path = self.get_path()?;

        self.output
            .as_ref()
            .map(|output| format!("{}/{}", path, output.filename))
    }

    #[allow(dead_code)]
    pub async fn get_url(&self, object_store: &Arc<ObjectStore>) -> Result<String, Error> {
        // Check if the task has an output
        let Some(path) = &self.get_file_path() else {
            return Err("Task has no output".into());
        };

        let path = match Path::parse(path) {
            Ok(p) => p,
            Err(e) => return Err(format!("Failed to parse path: {}", e).into()),
        };

        match **object_store {
            ObjectStore::S3(ref store) => {
                let url = store
                    .signed_url(Method::GET, &path, Duration::from_secs(600))
                    .await?;

                Ok(url.to_string())
            }
            ObjectStore::Local(_) => Ok(format!("file://{}", path)),
        }
    }

    /// Deletes the task from the object storage
    pub async fn delete_from_storage(
        &self,
        object_store: &Arc<ObjectStore>,
    ) -> Result<(), Error> {
        // Check if the task has an output
        let Some(path) = self.get_path() else {
            return Err("Task has no output".into());
        };

        let Some(outp) = &self.output else {
            return Err("Task has no output".into());
        };

        for path in [format!("{}/{}", path, outp.filename), path].iter() {
            let path = match Path::parse(path) {
                Ok(p) => p,
                Err(e) => return Err(format!("Failed to parse path: {}", e).into()),
            };

            // If the task has an output, delete it from the object store
            object_store.get().delete(&path).await?;
        }

        Ok(())
    }

    /// Delete the task from the database, this also consumes the task dropping it from memory
    pub async fn delete_from_db(self, pool: &PgPool) -> Result<(), Error> {
        sqlx::query!("DELETE FROM tasks WHERE task_id = $1", self.task_id,)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Deletes the task entirely, this includes deleting it from the object storage and the database
    /// This also consumes the task dropping it from memory
    #[allow(dead_code)] // Will be used in the near future
    pub async fn delete(
        self,
        pool: &PgPool,
        object_store: &Arc<ObjectStore>,
    ) -> Result<(), Error> {
        self.delete_from_storage(object_store).await?;
        self.delete_from_db(pool).await?;

        Ok(())
    }
}
