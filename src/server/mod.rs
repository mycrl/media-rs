mod websocket_flv;
mod http_flv;
mod rtmp;

use std::sync::Arc;
use crate::{
    config::Config,
    router::Router,
};

pub fn run(cfg: Arc<Config>) {
    let router = Router::new();

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
