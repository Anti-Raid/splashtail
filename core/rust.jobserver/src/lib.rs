pub mod embed;
pub mod poll;
pub mod spawn;

use indexmap::IndexMap;
use splashcore_rs::objectstore::ObjectStore;
use sqlx::{types::uuid::Uuid, PgPool};
use std::str::FromStr;
use std::time::Duration;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SpawnResponse {
    pub id: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Spawn {
    pub name: String,
    pub data: serde_json::Value,
    pub create: bool,
    pub execute: bool,
    pub id: Option<String>, // If create is false, this is required
    pub user_id: String,
}

/// Rust internal/special type to better serialize/speed up embed creation
#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct Statuses {
    pub level: String,
    pub msg: String,
    pub ts: f64,
    #[serde(rename = "botDisplayIgnore")]
    pub bot_display_ignore: Option<Vec<String>>,

    #[serde(flatten)]
    pub extra_info: IndexMap<String, serde_json::Value>,
}

pub struct Job {
    pub id: Uuid,
    pub name: String,
    pub output: Option<Output>,
    pub fields: IndexMap<String, serde_json::Value>,
    pub statuses: Vec<Statuses>,
    pub owner: Owner,
    pub expiry: Option<chrono::Duration>,
    pub state: String,
    pub resumable: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Owner {
    pub id: String,
    pub target_type: String,
}

impl FromStr for Owner {
    type Err = splashcore_rs::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(2, '/');
        let target_type = split.next().ok_or("Invalid owner.target_type")?;
        let id = split.next().ok_or("Invalid owner.id")?;

        Ok(Self {
            id: id.to_string(),
            target_type: target_type.to_string(),
        })
    }
}

impl From<String> for Owner {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Output {
    pub filename: String,
    pub segregated: bool,
}

/// JobCreateResponse is the response upon creation of a job
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct JobCreateResponse {
    /// The ID of the newly created task
    pub id: String,
}

impl Job {
    /// Fetches a task from the database based on id
    pub async fn from_id(id: Uuid, pool: &PgPool) -> Result<Self, splashcore_rs::Error> {
        let rec = sqlx::query!(
            "SELECT id, name, output, statuses, owner, expiry, state, created_at, fields, resumable FROM jobs WHERE id = $1 ORDER BY created_at DESC",
            id,
        )
        .fetch_one(pool)
        .await?;

        let mut statuses = Vec::new();

        for status in &rec.statuses {
            let status = serde_json::from_value::<Statuses>(status.clone())?;
            statuses.push(status);
        }

        let task = Job {
            id: rec.id,
            name: rec.name,
            output: rec
                .output
                .map(serde_json::from_value::<Output>)
                .transpose()?,
            fields: serde_json::from_value::<IndexMap<String, serde_json::Value>>(rec.fields)?,
            statuses,
            owner: rec.owner.into(),
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

    /// Fetches all jobs of a guild given guild id
    #[allow(dead_code)] // Will be used in the near future
    pub async fn from_guild(
        guild_id: serenity::all::GuildId,
        pool: &sqlx::PgPool,
    ) -> Result<Vec<Self>, splashcore_rs::Error> {
        let recs = sqlx::query!(
            "SELECT id, name, output, statuses, owner, expiry, state, created_at, fields, resumable FROM jobs WHERE owner = $1",
            format!("g/{}", guild_id)
        )
        .fetch_all(pool)
        .await?;

        let mut jobs = Vec::new();

        for rec in recs {
            let mut statuses = Vec::new();

            for status in &rec.statuses {
                let status = serde_json::from_value::<Statuses>(status.clone())?;
                statuses.push(status);
            }

            let job = Job {
                id: rec.id,
                name: rec.name,
                output: rec
                    .output
                    .map(serde_json::from_value::<Output>)
                    .transpose()?,
                fields: serde_json::from_value::<IndexMap<String, serde_json::Value>>(rec.fields)?,
                statuses,
                owner: rec.owner.into(),
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

            jobs.push(job);
        }

        Ok(jobs)
    }

    /// Returns all jobs with a specific guild ID and a specific task name
    pub async fn from_guild_and_name(
        guild_id: serenity::all::GuildId,
        name: &str,
        pool: &sqlx::PgPool,
    ) -> Result<Vec<Self>, splashcore_rs::Error> {
        let recs = sqlx::query!(
            "SELECT id, name, output, statuses, owner, expiry, state, created_at, fields, resumable FROM jobs WHERE owner = $1 AND name = $2",
            format!("g/{}", guild_id),
            name,
        )
        .fetch_all(pool)
        .await?;

        let mut jobs = Vec::new();

        for rec in recs {
            let mut statuses = Vec::new();

            for status in &rec.statuses {
                let status = serde_json::from_value::<Statuses>(status.clone())?;
                statuses.push(status);
            }

            let job = Job {
                id: rec.id,
                name: rec.name,
                output: rec
                    .output
                    .map(serde_json::from_value::<Output>)
                    .transpose()?,
                fields: serde_json::from_value::<IndexMap<String, serde_json::Value>>(rec.fields)?,
                statuses,
                owner: rec.owner.into(),
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

            jobs.push(job);
        }

        Ok(jobs)
    }

    pub fn format_owner_simplex(&self) -> String {
        format!(
            "{}/{}",
            self.owner.target_type.to_lowercase(),
            self.owner.id
        )
    }

    pub fn get_path(&self) -> Option<String> {
        if let Some(output) = &self.output {
            if output.segregated {
                Some(format!(
                    "{}/{}/{}",
                    self.format_owner_simplex(),
                    self.name,
                    self.id,
                ))
            } else {
                Some(format!("jobs/{}", self.id))
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
    pub async fn get_url(
        &self,
        object_store: &ObjectStore,
    ) -> Result<String, splashcore_rs::Error> {
        // Check if the job has an output
        let Some(path) = &self.get_file_path() else {
            return Err("Job has no output".into());
        };

        Ok(object_store.get_url(path, Duration::from_secs(600)))
    }

    /// Deletes the job from the object storage
    pub async fn delete_from_storage(
        &self,
        client: &reqwest::Client,
        object_store: &ObjectStore,
    ) -> Result<(), splashcore_rs::Error> {
        // Check if the job has an output
        let Some(path) = self.get_path() else {
            return Err("Job has no output".into());
        };

        let Some(outp) = &self.output else {
            return Err("Job has no output".into());
        };

        object_store
            .delete(client, &format!("{}/{}", path, outp.filename))
            .await?;

        Ok(())
    }

    /// Delete the job from the database, this also consumes the job dropping it from memory
    pub async fn delete_from_db(self, pool: &PgPool) -> Result<(), splashcore_rs::Error> {
        sqlx::query!("DELETE FROM jobs WHERE id = $1", self.id,)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Deletes the job entirely, this includes deleting it from the object storage and the database
    /// This also consumes the job dropping it from memory
    #[allow(dead_code)] // Will be used in the near future
    pub async fn delete(
        self,
        pool: &PgPool,
        client: &reqwest::Client,
        object_store: &ObjectStore,
    ) -> Result<(), splashcore_rs::Error> {
        self.delete_from_storage(client, object_store).await?;
        self.delete_from_db(pool).await?;

        Ok(())
    }
}
