mod websocket;
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
        log::info!("rtmp server listen: {}", cfg.listen);
    }

    if let Some(cfg) = &cfg.proto.ws {
        tokio::spawn(websocket::run(cfg.clone(), router));
        log::info!("websocket server listen: {}", cfg.listen);
    }
}
