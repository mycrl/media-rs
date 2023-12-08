mod http_flv;
mod rtmp;
mod websocket_flv;

use crate::{config::Config, router::Router};
use std::sync::Arc;

pub fn run(cfg: Arc<Config>) {
    let router = Arc::new(Router::default());

    if let Some(cfg) = &cfg.proto.rtmp {
        tokio::spawn(rtmp::run(cfg.clone(), router.clone()));
        log::info!("rtmp server listening: {}", cfg.listen);
    }

    if let Some(cfg) = &cfg.proto.websocket_flv {
        tokio::spawn(websocket_flv::run(cfg.clone(), router.clone()));
        log::info!("websocket flv server listening: {}", cfg.listen);
    }

    if let Some(cfg) = &cfg.proto.http_flv {
        tokio::spawn(http_flv::run(cfg.clone(), router));
        log::info!("http flv server listening: {}", cfg.listen);
    }
}
