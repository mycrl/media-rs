use tokio_tungstenite::tungstenite::protocol::*;
use serde::Deserialize;
use clap::Parser;
use std::{
    net::SocketAddr,
    fs::*,
};

#[derive(Deserialize, Debug, Clone)]
pub struct Rtmp {
    #[serde(default = "Rtmp::listen")]
    pub listen: SocketAddr,
    #[serde(default = "Rtmp::band_width")]
    pub band_width: usize,
}

impl Rtmp {
    fn listen() -> SocketAddr {
        "127.0.0.1:1935".parse().unwrap()
    }

    fn band_width() -> usize {
        5000000
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Ws {
    #[serde(default = "Ws::listen")]
    pub listen: SocketAddr,
    /// The size of the send queue. You can use it to turn on/off the
    /// backpressure features. None means here that the size of the queue is
    /// unlimited. The default value is the unlimited queue.
    #[serde(default)]
    pub max_send_queue: Option<usize>,
    /// The maximum size of a message. None means no size limit. The default
    /// value is 64 MiB which should be reasonably big for all normal use-cases
    /// but small enough to prevent memory eating by a malicious user.
    #[serde(default)]
    pub max_message_size: Option<usize>,
    /// The maximum size of a single message frame. None means no size limit.
    /// The limit is for frame payload NOT including the frame header. The
    /// default value is 16 MiB which should be reasonably big for all normal
    /// use-cases but small enough to prevent memory eating by a malicious
    /// user.
    #[serde(default)]
    pub max_frame_size: Option<usize>,
    /// When set to true, the server will accept and handle unmasked frames
    /// from the client. According to the RFC 6455, the server must close the
    /// connection to the client in such cases, however it seems like there are
    /// some popular libraries that are sending unmasked frames, ignoring the
    /// RFC. By default this option is set to false, i.e. according to RFC
    /// 6455.
    #[serde(default)]
    pub accept_unmasked_frames: bool,
}

impl Ws {
    fn listen() -> SocketAddr {
        "127.0.0.1:8080".parse().unwrap()
    }

    pub fn get_config(&self) -> WebSocketConfig {
        WebSocketConfig {
            max_send_queue: self.max_send_queue,
            max_message_size: self.max_message_size,
            max_frame_size: self.max_frame_size,
            accept_unmasked_frames: self.accept_unmasked_frames,
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct Proto {
    pub rtmp: Option<Rtmp>,
    pub ws: Option<Ws>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl LogLevel {
    pub fn as_level(&self) -> log::Level {
        match *self {
            Self::Error => log::Level::Error,
            Self::Debug => log::Level::Debug,
            Self::Trace => log::Level::Trace,
            Self::Warn => log::Level::Warn,
            Self::Info => log::Level::Info,
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct Log {
    /// log level
    ///
    /// An enum representing the available verbosity levels of the logger.
    #[serde(default)]
    pub level: LogLevel,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub proto: Proto,
    #[serde(default)]
    pub log: Log,
}

#[derive(Parser)]
#[command(
    about = env!("CARGO_PKG_DESCRIPTION"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
)]
struct Cli {
    /// specify the configuration file path.
    #[arg(long)]
    config: Option<String>,
}

impl Config {
    /// Load command line parameters, if the configuration file path is
    /// specified, the configuration is read from the configuration file,
    /// otherwise the default configuration is used.
    pub fn load() -> Self {
        toml::from_str(
            &Cli::parse()
                .config
                .and_then(|path| read_to_string(path).ok())
                .unwrap_or("".to_string()),
        )
        .unwrap()
    }
}
