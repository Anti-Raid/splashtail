use std::sync::Arc;
use fred::interfaces::PubsubInterface;
use serde::{Serialize, Deserialize};

/// This is the fundemental primitive atop which the whole of Anti-Raids scales
pub struct IpcClient {
    pub redis_pool: fred::clients::RedisPool,
    pub shard_manager: Arc<serenity::all::ShardManager>,
    pub mewld_args: Arc<crate::ipc::argparse::MewldCmdArgs>,
}

/*
Scope     string         `json:"scope"`
	Action    string         `json:"action"`
	Args      map[string]any `json:"args,omitempty"`
	CommandId string         `json:"command_id,omitempty"`
	Output    any            `json:"output,omitempty"`
	Data      map[string]any `json:"data,omitempty"` // Used in action logs */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherCmd {
    scope: String,
    action: String,
    args: Option<serde_json::Value>,
    command_id: Option<String>,
    output: Option<serde_json::Value>,
    data: Option<serde_json::Value>,
}

impl IpcClient {
    /// Publishes a message to the redis IPC channel via the standard launchercmd
    pub async fn publish_ipc_launchercmd(&self, cmd: LauncherCmd) -> Result<(), crate::Error> {
        let cmd = serde_json::to_string(&cmd)?;

        let conn = self.redis_pool.next();
        conn.publish(self.mewld_args.mewld_redis_channel.clone(), cmd).await?;

        Ok(())
    }

    /// Publishes a launch_next command
    pub async fn publish_ipc_launch_next(&self) -> Result<(), crate::Error> {
        let cmd = LauncherCmd {
            scope: "launcher".to_string(),
            action: "launch_next".to_string(),
            args: Some(serde_json::json!({
                "id": self.mewld_args.cluster_id,
            })),
            command_id: None,
            output: None,
            data: None,
        };

        self.publish_ipc_launchercmd(cmd).await
    }
}