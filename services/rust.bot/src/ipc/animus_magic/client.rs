use super::bot::BotAnimusMessage;
use crate::{ipc::argparse::MEWLD_ARGS, Error};
use botox::cache::CacheHttpImpl;
use fred::{
    interfaces::{ClientLike, EventInterface, PubsubInterface},
    prelude::Builder,
    types::RedisValue,
};
use futures_util::future::FutureExt;
use splashcore_rs::animusmagic::client::{ClientRequest, UnderlyingClient};
use splashcore_rs::animusmagic::protocol::{
    get_payload_meta, serialize_data, AnimusErrorResponse, AnimusTarget,
};
use std::sync::Arc;

pub struct ClientData {
    pub data: Arc<base_data::Data>,
    pub cache_http: CacheHttpImpl,
}

pub struct ClientDataWrapper(pub Arc<ClientData>);

pub struct AnimusMagicClient {
    pub underlying_client: Arc<UnderlyingClient<ClientDataWrapper>>,
    pub allow_all: bool,
}

async fn publish(data: Arc<ClientData>, payload: Vec<u8>) -> Result<(), Error> {
    // Convert payload to redis value
    let payload = RedisValue::Bytes(payload.into());

    match data
        .data
        .redis_pool
        .next()
        .publish(MEWLD_ARGS.animus_magic_channel.as_str(), payload)
        .await
    {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("Failed to publish response to redis: {}", e).into()),
    }
}

async fn on_request(
    state: Arc<ClientData>,
    resp: ClientRequest,
) -> Result<Vec<u8>, AnimusErrorResponse> {
    let parsed_resp = resp
        .parse::<BotAnimusMessage>()
        .map_err(|e| AnimusErrorResponse {
            message: e.to_string(),
            context: "on_request [resp.parse]".to_string(),
        })?;

    let resp = parsed_resp.response(state).await?;

    Ok(serialize_data(&resp)?)
}

impl AnimusMagicClient {
    pub fn new(data: ClientData) -> Self {
        let underlying_client = Arc::new(UnderlyingClient::new(
            AnimusTarget::Bot,
            MEWLD_ARGS.cluster_id,
            ClientDataWrapper(Arc::new(data)),
            Box::new(move |data, payload| publish(data.0.clone(), payload).boxed()),
            Some(Box::new(move |data, payload| {
                on_request(data.0.clone(), payload).boxed()
            })),
            None,
            None,
        ));

        Self {
            underlying_client,
            allow_all: false,
        }
    }

    /// Starts listening to animus magic messages
    ///
    /// These messages will then be passed on to the underlying client
    pub async fn listen(&self) -> ! {
        // Subscribes to the redis IPC channels we need to subscribe to
        let cfg = self
            .underlying_client
            .state
            .0
            .data
            .redis_pool
            .client_config();

        let subscriber = Builder::from_config(cfg).build_subscriber_client().unwrap();

        subscriber.connect();
        subscriber.wait_for_connect().await.unwrap();

        self.underlying_client
            .state
            .0
            .data
            .redis_pool
            .connect_pool();

        let mut message_stream = subscriber.message_rx();

        subscriber.manage_subscriptions();

        let _: () = subscriber
            .subscribe(MEWLD_ARGS.animus_magic_channel.as_str())
            .await
            .unwrap();

        while let Ok(message) = message_stream.recv().await {
            log::debug!("Got message on channel {}", message.channel);
            let binary = match message.value {
                RedisValue::Bytes(s) => s,
                RedisValue::String(s) => s.into(),
                _ => {
                    log::warn!(
                        "Invalid message recieved on channel [wanted binary, but got text] {}",
                        message.channel
                    );
                    continue;
                }
            };

            // Parse the payload
            let meta = match get_payload_meta(&binary) {
                Ok(meta) => meta,
                Err(e) => {
                    log::warn!(
                        "Invalid message recieved on channel {} [metadata extract error: {}]",
                        message.channel,
                        e
                    );
                    continue;
                }
            };

            if !self.allow_all && !self.underlying_client.filter(&meta) {
                continue;
            }

            let payload = binary[meta.payload_offset..].to_vec();
            let underlying_client = self.underlying_client.clone();
            tokio::spawn(async move {
                let underlying_client = underlying_client;
                if let Err(e) = underlying_client.handle(meta, payload).await {
                    log::error!("Error handling animus magic message: {}", e);
                };
            });
        }

        unreachable!("IPC listener exited");
    }
}
