use super::protocol::{
    create_payload, new_command_id, serialize_data, AnimusErrorResponse, AnimusMessageMetadata,
    AnimusOp, AnimusTarget, WILDCARD_CLUSTER_ID,
};
use crate::Error;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub enum ClientError {
    Timeout,
    OpError,
    RecievedMoreResponsesThanExpected,
    UnknownOp,
    NoResponse,
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ClientError::Timeout => write!(f, "Request timed out"),
            ClientError::OpError => write!(f, "Received an OpError response"),
            ClientError::RecievedMoreResponsesThanExpected => {
                write!(f, "Received more responses than expected")
            }
            ClientError::UnknownOp => write!(f, "Received an unknown op type"),
            ClientError::NoResponse => write!(f, "Received no response"),
        }
    }
}

impl std::error::Error for ClientError {}

pub trait AnimusResponse {
    fn target(&self) -> AnimusTarget;
}

pub trait AnimusMessage {
    fn target(&self) -> AnimusTarget;
}

pub trait SerializableAnimusResponse: AnimusResponse + Serialize {}
pub trait SerializableAnimusMessage: AnimusMessage + Serialize {}

/// A ClientResponse contains the response from animus magic
pub struct ClientResponse {
    /// Metadata
    pub meta: AnimusMessageMetadata,

    /// The raw payload
    pub raw_payload: Vec<u8>,
}

pub struct ParsedClientResponse<Response: SerializableAnimusResponse + for<'a> Deserialize<'a>> {
    pub err: Option<AnimusErrorResponse>,
    pub resp: Option<Response>,
    pub client_resp: Option<ClientResponse>,
}

impl ClientResponse {
    pub fn parse<Response: SerializableAnimusResponse + for<'a> Deserialize<'a>>(
        self,
    ) -> Result<ParsedClientResponse<Response>, crate::Error> {
        match self.meta.op {
            AnimusOp::Error => {
                Ok(ParsedClientResponse {
                    err: Some(serde_cbor::from_slice(&self.raw_payload)?),
                    resp: None, // We may support error + response in the future
                    client_resp: Some(self),
                })
            }
            AnimusOp::Response => Ok(ParsedClientResponse {
                err: None,
                resp: Some(serde_cbor::from_slice(&self.raw_payload)?),
                client_resp: Some(self),
            }),
            _ => Err(ClientError::UnknownOp.into()),
        }
    }
}

/// A ClientRequest contains the request from animus magic
pub struct ClientRequest {
    /// Metadata
    pub meta: AnimusMessageMetadata,

    /// The raw payload
    pub raw_payload: Vec<u8>,
}

impl ClientRequest {
    pub fn parse<T: Serialize + for<'a> Deserialize<'a>>(&self) -> Result<T, crate::Error> {
        Ok(serde_cbor::from_slice(&self.raw_payload)?)
    }
}

/// RequestOptions stores the options for a request
#[derive(Default)]
pub struct RequestOptions {
    /// The cluster id to send to, must be set, also ExpectedResponseCount must be set if wildcard
    pub cluster_id: u16,

    /// Must be set if wildcard. this is the number of responses expected
    pub expected_response_count: usize,

    /// If unset, will be randomly generated
    pub command_id: String,

    /// Must be set
    pub to: AnimusTarget,

    /// Must be set
    pub op: AnimusOp,

    /// If true, will ignore OpError responses
    pub ignore_op_error: bool,
}

impl RequestOptions {
    pub fn parse(&mut self) -> Result<(), crate::Error> {
        if self.expected_response_count == 0 {
            if self.cluster_id == WILDCARD_CLUSTER_ID {
                return Err("Expected response count is not set".into());
            } else {
                self.expected_response_count = 1;
            }
        }

        if self.command_id.is_empty() {
            self.command_id = new_command_id();
        }

        Ok(())
    }
}

pub struct NotifyWrapper {
    pub chan: tokio::sync::mpsc::Sender<ClientResponse>,
    pub expected_count: usize,
    pub response_count: std::sync::atomic::AtomicUsize,
}

pub type OnRequest<T> = Box<
    dyn Send
        + Sync
        + Fn(T, ClientRequest) -> BoxFuture<'static, Result<Vec<u8>, AnimusErrorResponse>>,
>;

pub type OnResponse<T> =
    Box<dyn Send + Sync + Fn(T, ClientResponse) -> BoxFuture<'static, Result<(), Error>>>;

pub type OnMiddleware<T> = Box<
    dyn Send
        + Sync
        + for<'a> Fn(T, &'a AnimusMessageMetadata, &[u8]) -> BoxFuture<'a, Result<bool, Error>>,
>;

/// Publisher is a function that publishes a message to the next available connection
pub type Publisher<T> =
    Box<dyn Send + Sync + Fn(T, Vec<u8>) -> BoxFuture<'static, Result<(), crate::Error>>>;

/// This is the underlying client for all animus magic applications
pub struct UnderlyingClient<T: Clone> {
    pub state: T,

    pub rx_map: Arc<dashmap::DashMap<String, NotifyWrapper>>,
    pub identity: AnimusTarget,
    pub cluster_id: u16,

    /// The publisher function. This function should publish the message to the next available connection
    pub publish: Publisher<T>,

    /// On request function, if set, will be called upon recieving op of type OpRequest
    pub on_request: Option<OnRequest<T>>,

    /// On response function, if set, will be called upon recieving op of type OpResponse
    pub on_response: Option<OnResponse<T>>,

    /// Middleware function, will be called regardless of the op
    ///
    /// If bool is false, the message will be ignored/dropped for further processing
    pub on_middleware: Option<OnMiddleware<T>>,

    /// The process id of the client
    pub pid: String,
}

impl<T: Clone> UnderlyingClient<T> {
    /// New creates a new client
    pub fn new(
        identity: AnimusTarget,
        cluster_id: u16,
        state: T,
        publish: Publisher<T>,
        on_request: Option<OnRequest<T>>,
        on_response: Option<OnResponse<T>>,
        on_middleware: Option<OnMiddleware<T>>,
    ) -> Self {
        Self {
            state,
            rx_map: Arc::new(dashmap::DashMap::new()),
            identity,
            cluster_id,
            publish,
            on_request,
            on_response,
            on_middleware,
            pid: std::process::id().to_string(),
        }
    }

    /// Filter determines whether or not the message should be processed
    pub fn filter(&self, meta: &AnimusMessageMetadata) -> bool {
        // If the message is not to us, ignore it
        if self.identity != meta.to && meta.to != AnimusTarget::Wildcard {
            return false;
        }

        // If the target cluster id
        if self.cluster_id != meta.cluster_id_to && meta.cluster_id_to != WILDCARD_CLUSTER_ID {
            return false;
        }

        true
    }

    /// Handle handles a request, processing filters, responding to probes and dispatching the event to notifiers
    pub async fn handle(
        &self,
        meta: AnimusMessageMetadata,
        payload: Vec<u8>,
    ) -> Result<(), crate::Error> {
        // First, check for and run any middleware
        if let Some(on_middleware) = &self.on_middleware {
            if !on_middleware(self.state.clone(), &meta, &payload).await? {
                return Ok(());
            }
        }

        #[allow(unreachable_patterns)]
        match meta.op {
            AnimusOp::Probe => {
                self.create_payload::<String>(
                    &meta.command_id,
                    meta.cluster_id_from,
                    meta.from,
                    AnimusOp::Response,
                    &self.pid,
                )
                .map_err(|e| format!("Failed to create payload for probe response: {}", e))?;

                (self.publish)(self.state.clone(), payload)
                    .await
                    .map_err(|e| format!("Failed to publish probe response: {}", e))?;
            }
            AnimusOp::Request => {
                if let Some(on_request) = &self.on_request {
                    match on_request(
                        self.state.clone(),
                        ClientRequest {
                            meta: meta.clone(),
                            raw_payload: payload,
                        },
                    )
                    .await
                    {
                        Ok(resp) => {
                            let payload = self.create_payload_raw(
                                &meta.command_id,
                                meta.cluster_id_from,
                                meta.from,
                                AnimusOp::Response,
                                resp,
                            );

                            (self.publish)(self.state.clone(), payload)
                                .await
                                .map_err(|e| format!("Failed to publish response: {}", e))?;
                        }
                        Err(e) => {
                            self.error(&meta.command_id, e, meta.cluster_id_from, meta.from)
                                .await?;
                        }
                    }
                }
            }
            AnimusOp::Response | AnimusOp::Error => {
                if let Some(on_response) = &self.on_response {
                    on_response(
                        self.state.clone(),
                        ClientResponse {
                            meta: meta.clone(),
                            raw_payload: payload.clone(),
                        },
                    )
                    .await?;
                }

                if let Some(notifier) = self.rx_map.get(&meta.command_id) {
                    let wrapper = notifier.value();

                    let new_count = wrapper
                        .response_count
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                    if wrapper.expected_count != 0 && new_count >= wrapper.expected_count {
                        self.close_notifier(&meta.command_id);
                        return Err(ClientError::RecievedMoreResponsesThanExpected.into());
                    }

                    let _ = wrapper.chan.send(ClientResponse {
                        meta: meta.clone(),
                        raw_payload: payload,
                    });

                    if wrapper.expected_count != 0 && new_count == wrapper.expected_count {
                        self.close_notifier(&meta.command_id);
                    }
                }
            }
            _ => {
                return Err(ClientError::UnknownOp.into());
            }
        }

        Ok(())
    }

    /// Creates a payload based on the clients and returns a byte vector
    pub fn create_payload<U: Serialize>(
        &self,
        cmd_id: &str,
        cluster_id_to: u16,
        to: AnimusTarget,
        op: AnimusOp,
        msg: &U,
    ) -> Result<Vec<u8>, Error> {
        Ok(create_payload(
            cmd_id,
            self.identity,
            self.cluster_id,
            cluster_id_to,
            to,
            op,
            serialize_data(msg)?,
        ))
    }

    /// Creates a payload based on the clients and returns a byte vector
    ///
    /// Unlike create_payload, this accepts a raw Vec<u8> instead of a serializable object
    pub fn create_payload_raw(
        &self,
        cmd_id: &str,
        cluster_id_to: u16,
        to: AnimusTarget,
        op: AnimusOp,
        msg: Vec<u8>,
    ) -> Vec<u8> {
        create_payload(
            cmd_id,
            self.identity,
            self.cluster_id,
            cluster_id_to,
            to,
            op,
            msg,
        )
    }

    /// CreateNotifier adds a notifier to the map and returns a `Reciever` for the notifier
    ///
    /// This channel will receive the response for the given command id
    pub fn create_notifier(
        &self,
        cmd_id: &str,
        expected_response_count: usize,
    ) -> Result<tokio::sync::mpsc::Receiver<ClientResponse>, Error> {
        let (tx, rx) = tokio::sync::mpsc::channel(expected_response_count);

        self.rx_map.insert(
            cmd_id.to_string(),
            NotifyWrapper {
                chan: tx,
                expected_count: expected_response_count,
                response_count: AtomicUsize::new(0),
            },
        );

        Ok(rx)
    }

    /// CloseNotifier closes the notifier for the given command id
    pub fn close_notifier(&self, cmd_id: &str) {
        self.rx_map.remove(cmd_id);
    }

    pub async fn gather_responses(
        &self,
        request_opts: &RequestOptions,
        timeout: Duration,
        mut notifier: tokio::sync::mpsc::Receiver<ClientResponse>,
    ) -> Result<Vec<ClientResponse>, Error> {
        let mut responses = Vec::new();

        let mut ticker = tokio::time::interval(timeout);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    self.close_notifier(&request_opts.command_id);
                    return Err(ClientError::Timeout.into())
                }
                Some(resp) = notifier.recv() => {
                    if resp.meta.op == AnimusOp::Error && !request_opts.ignore_op_error {
                        responses.push(resp);
                        self.close_notifier(&request_opts.command_id);
                        return Err(ClientError::OpError.into())
                    }

                    if responses.len() >= request_opts.expected_response_count {
                        responses.push(resp);
                        self.close_notifier(&request_opts.command_id);
                        return Ok(responses)
                    }
                }
            }
        }
    }

    /// Request creates a new request and waits for a response until either timeout or response
    pub async fn request<U: SerializableAnimusMessage>(
        &self,
        mut opts: RequestOptions,
        timeout: Duration,
        msg: U,
    ) -> Result<Vec<ClientResponse>, crate::Error> {
        opts.parse()?;

        let payload = match self.create_payload::<U>(
            &opts.command_id,
            opts.cluster_id,
            opts.to,
            opts.op,
            &msg,
        ) {
            Ok(payload) => payload,
            Err(e) => return Err(e),
        };

        let rx = self.create_notifier(&opts.command_id, opts.expected_response_count)?;

        (self.publish)(self.state.clone(), payload).await?;

        self.gather_responses(&opts, timeout, rx).await
    }

    /// request_one is a helper function that sends a request and waits for a single response
    /// parsing it afterwards
    pub async fn request_one<
        U: SerializableAnimusMessage,
        R: SerializableAnimusResponse + for<'a> Deserialize<'a>,
    >(
        &self,
        opts: RequestOptions,
        timeout: Duration,
        msg: U,
    ) -> Result<ParsedClientResponse<R>, crate::Error> {
        let responses = self.request(opts, timeout, msg).await?;

        if responses.len() != 1 {
            return Err(ClientError::NoResponse.into());
        }

        let first_response = responses.into_iter().next().unwrap();
        first_response.parse()
    }

    /// Helper method to send an error response
    pub async fn error(
        &self,
        command_id: &str,
        data: AnimusErrorResponse,
        cluster_id_to: u16,
        to: AnimusTarget,
    ) -> Result<(), crate::Error> {
        let Ok(payload) = self.create_payload::<AnimusErrorResponse>(
            command_id,
            cluster_id_to,
            to,
            AnimusOp::Error,
            &data,
        ) else {
            return Err("Failed to create payload for error message".into());
        };

        (self.publish)(self.state.clone(), payload).await
    }
}
