/// The `AnimusMagicClientExt` trait provides a set of extension methods for clients
use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::Error;
use std::sync::Arc;
use crate::animusmagic_protocol::{AnimusTarget, AnimusOp, AnimusErrorResponse, create_payload, new_command_id};
use tokio::sync::mpsc::Sender;

pub enum AnimusAnyResponse<Response>
where Response: Serialize + for<'a> Deserialize<'a> {
    Response(Response),
    Error(AnimusErrorResponse),
}

#[allow(async_fn_in_trait)] // It's our own code
pub trait AnimusMagicClientExt<Response>
where Response: Serialize + for<'a> Deserialize<'a> {
    /// Returns the map of command ids to response senders
    fn rx_map(&self) -> Arc<dashmap::DashMap<String, Sender<AnimusAnyResponse<Response>>>>;

    /// Returns who the client is/from
    fn from(&self) -> AnimusTarget;

    /// Returns the cluster id of the client
    fn cluster_id(&self) -> u16;

    /// Creates a payload based on the clients and returns a byte vector
    fn create_payload_simplex<T: Serialize>(
        &self,
        cmd_id: &str,
        to: AnimusTarget,
        op: AnimusOp,
        msg: &T,
    ) -> Result<Vec<u8>, Error> {
        create_payload::<T>(
            cmd_id,
            self.from(),
            self.cluster_id(),
            to,
            op,
            msg,
        )
    }

    /// Publish via the next available connection
    async fn publish_next(&self, payload: Vec<u8>) -> Result<(), Error>;

    /// Request creates a new request and waits for a response until either timeout or response
    async fn request<T: Serialize>(
        &self,
        target: AnimusTarget,
        msg: T,
        timeout: Duration,
    ) -> Result<AnimusAnyResponse<Response>, crate::Error> {
        let cmd_id = new_command_id();

        let payload = match self.create_payload_simplex::<T>(
            &cmd_id,
            target,
            AnimusOp::Request,
            &msg,
        ) {
            Ok(payload) => payload,
            Err(e) => return Err(e),
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        self.rx_map().insert(cmd_id.clone(), tx);

        self.publish_next(payload).await?;

        let resp = match tokio::time::timeout(timeout, rx.recv()).await {
            Ok(resp) => resp,
            Err(_) => return Err("Request timed out".into()),
        };

        match resp {
            Some(resp) => Ok(resp),
            None => Err("Failed to get response".into()),
        }
    }    

    /// Helper method to send an error response
    async fn error(
        &self,
        command_id: &str,
        data: AnimusErrorResponse,
        to: AnimusTarget,
    ) -> Result<(), crate::Error> {
        let Ok(payload) = self.create_payload_simplex::<AnimusErrorResponse>(
            command_id,
            to,
            AnimusOp::Error,
            &data,
        ) else {
            return Err("Failed to create payload for error message".into());
        };

        self.publish_next(payload).await
    }
}
