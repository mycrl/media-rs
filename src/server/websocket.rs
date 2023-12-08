use std::{net::SocketAddr, sync::Arc};

use crate::{proto::websocket::*,router::*,config,};

use anyhow::Result;
use futures_util::sink::SinkExt;
use tokio_tungstenite::tungstenite::Message;

async fn fork_socket(
    addr: SocketAddr,
    cfg: Arc<config::Ws>,
    router: Arc<Router>,
    socket: TcpStream,
) {
    if let Ok((mut stream, query)) =
        accept(socket, Some(cfg.get_config())).await
    {
        log::info!(
            "websocket connection name: {}, key: {}",
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

    log::info!("websocket connection close: {}", addr);
}

pub async fn run(cfg: config::Ws, router: Arc<Router>) -> Result<()> {
    let cfg = Arc::new(cfg);
    let listener = TcpListener::bind(&cfg.listen).await?;
    while let Ok((socket, addr)) = listener.accept().await {
        log::info!("websocket connection pull: {}", addr);
        tokio::spawn(fork_socket(addr, cfg.clone(), router.clone(), socket));
    }

    Ok(())
}
