pub mod taskpoll;

use std::str::FromStr;

/**
 * export interface Task {
  task_id: string;
  task_name: string;
  output?: TaskOutput;
  task_info?: TaskInfo;
  statuses: { [key: string]: any}[];
  task_for?: TaskFor;
  expiry?: any /* time.Duration */;
  state: string;
  created_at: string /* RFC3339 */;
}

/**
 * TaskFor is a struct containing the internal representation of who a task is for
 */
export interface TaskFor {
  id: string;
  target_type: string;
}
/**
 * TaskOutput is the output of a task
 */
export interface TaskOutput {
  filename: string;
  segregated: boolean; // If this flag is set, then the stored output will be stored in $taskForSimplexFormat/$taskName/$taskId/$filename instead of $taskId/$filename
}
/**
 * Information on a task
 */
export interface TaskInfo {
  name: string;
  task_for?: TaskFor;
  task_fields: any;
  expiry: any /* time.Duration */;
  valid: boolean;
}
 */
#[derive(Clone)]
pub struct Task {
    pub task_id: sqlx::types::uuid::Uuid,
    pub task_name: String,
    pub output: Option<TaskOutput>,
    pub task_info: TaskInfo,
    pub statuses: Vec<serde_json::Value>,
    pub task_for: Option<TaskFor>,
    pub expiry: Option<chrono::Duration>,
    pub state: String,
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

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct TaskInfo {
    pub name: String,
    pub task_for: Option<TaskFor>,
    pub task_fields: serde_json::Value,
    pub expiry: Option<u64>,
    pub valid: bool,
}