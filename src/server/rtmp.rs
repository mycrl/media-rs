use std::{net::SocketAddr, sync::Arc};

use crate::{
    config,
    flv::FlvFrame,
    proto::rtmp::{Rtmp, RtmpObserver},
    router::{Router, RouterSender},
};

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

pub struct Observer {
    router: Arc<Router>,
    sender: Option<RouterSender>,
    app: Option<String>,
    addr: SocketAddr,
}

impl Observer {
    fn new(addr: SocketAddr, router: Arc<Router>) -> Self {
        Self {
            sender: None,
            app: None,
            router,
            addr,
        }
    }
}

#[async_trait]
impl RtmpObserver for Observer {
    async fn guard(&mut self, app: &str, key: &str) {
        log::info!(
            "rtmp publish stream addr: {}, name: {}, key: {}",
            self.addr,
            app,
            key
        );

        let sender = self.router.get_sender(&self.addr, app).await;
        let _ = self.sender.insert(sender);
        let _ = self.app.insert(app.to_string());
    }

    async fn data_frame(&mut self, buf: Bytes) {
        if let Some(sender) = &mut self.sender {
            sender.send(FlvFrame::Script, 0, buf).await;
        }
    }

    async fn audio_data(&mut self, timestamp: u32, buf: Bytes) {
        if let Some(sender) = &mut self.sender {
            sender.send(FlvFrame::Audio, timestamp, buf).await;
        }
    }

    async fn video_data(&mut self, timestamp: u32, buf: Bytes) {
        if let Some(sender) = &mut self.sender {
            sender.send(FlvFrame::Video, timestamp, buf).await;
        }
    }
}

async fn fork_socket(addr: SocketAddr, mut socket: TcpStream, router: Arc<Router>) {
    let mut buf = [0u8; 5120];
    let mut rtmp = Rtmp::new(Observer::new(addr, router.clone()));
    while let Ok(size) = socket.read(&mut buf).await {
        if size == 0 {
            break;
        }

        if let Ok(bytes) = rtmp.process(&buf[..size]).await {
            if !bytes.is_empty() && socket.write_all(&bytes).await.is_err() {
                break;
            }
        } else {
            break;
        }
    }

    log::info!("rtmp connection unpublish : {}", addr);
    router.remove(&addr).await;
}

pub async fn run(cfg: config::Rtmp, router: Arc<Router>) -> Result<()> {
    let listener = TcpListener::bind(cfg.listen).await?;
    while let Ok((socket, addr)) = listener.accept().await {
        log::info!("rtmp connection publish: {}", addr);
        tokio::spawn(fork_socket(addr, socket, router.clone()));
    }

    Ok(())
}
