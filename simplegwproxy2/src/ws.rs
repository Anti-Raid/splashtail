use tokio::net::TcpListener;
use futures_util::{SinkExt, StreamExt};

pub async fn start_ws() -> Result<(), crate::Error> {
    let listener = TcpListener::bind(
        format!("127.0.0.1:{}", crate::config::CONFIG.simple_gateway_proxy.port)
    ).await?;

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = tokio_websockets::ServerBuilder::new()
        .accept(stream)
        .await;

        if let Err(e) = ws_stream {
            log::error!("Failed to accept client: {}", e);
            continue;
        }

        let mut ws_stream = ws_stream.unwrap();

        tokio::spawn(async move {
            // Just an echo server, really
            if let Err(e) = connection(ws_stream).await {
                log::error!("Failed to handle connection: {}", e);
            }
        });
    }

    Ok(())
}

pub async fn connection(ws_stream: tokio_websockets::WebSocketStream<tokio::net::TcpStream>) -> Result<(), crate::Error> {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(Ok(msg)) = ws_receiver.next().await {
        if msg.is_text() || msg.is_binary() {
            ws_sender.send(msg).await?;
        }
    }

    Ok(())
}