mod server;
mod format;
mod config;
mod router;
mod proto;

use config::Config;
use std::{
    future::pending,
    sync::Arc,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Arc::new(Config::load());
    simple_logger::init_with_level(cfg.log.level.as_level())?;
    server::run(cfg);
    pending().await
}
