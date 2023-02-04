use tokio_tungstenite::tungstenite::Message;
use futures_util::sink::SinkExt;
use anyhow::Result;
use tokio::net::*;
use std::{
    net::SocketAddr,
    sync::Arc,
};

use crate::{
    config::WebSocketFlv,
    proto::websocket::*,
    router::*,
};

async fn fork_socket(
    addr: SocketAddr,
    cfg: Arc<WebSocketFlv>,
    router: Arc<Router>,
    socket: TcpStream,
) {
    if let Ok((mut stream, query)) =
        accept(socket, Some(cfg.get_config())).await
    {
        log::info!(
            "websocket flv connection name: {}, key: {}",
            query.name,
            query.key
        );

        if let Some(mut reader) = router.get_receiver(&addr, &query.name).await
        {
            while let Some(buf) = reader.read().await {
                if stream.send(Message::Binary(buf)).await.is_err() {
                    break;
                }
            }
        }
    }

    log::info!("websocket flv connection close: {}", addr);
}

pub async fn run(cfg: WebSocketFlv, router: Arc<Router>) -> Result<()> {
    let cfg = Arc::new(cfg);
    let listener = TcpListener::bind(&cfg.listen).await?;
    while let Ok((socket, addr)) = listener.accept().await {
        log::info!("websocket flv connection pull: {}", addr);
        tokio::spawn(fork_socket(addr, cfg.clone(), router.clone(), socket));
    }

    Ok(())
}
