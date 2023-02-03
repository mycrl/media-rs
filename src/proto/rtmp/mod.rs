mod session;

use std::borrow::Cow;
use async_trait::*;
use bytes::Bytes;
use session::*;
use rml_rtmp::{
    handshake::HandshakeProcessResult::*,
    handshake::*,
};

#[async_trait]
pub trait Observer: Send {
    async fn guard(&mut self, app: &str, key: &str);
    async fn data_frame(&mut self, buf: Bytes);
    async fn audio_data(&mut self, timestamp: u32, buf: Bytes);
    async fn video_data(&mut self, timestamp: u32, buf: Bytes);
}

pub struct Rtmp {
    handshake_state: bool,
    handshake: Handshake,
    session: Session,
}

impl Rtmp {
    pub fn new<T>(observer: T) -> Self
    where
        T: Observer + 'static,
    {
        Self {
            handshake_state: false,
            handshake: Handshake::new(PeerType::Server),
            session: Session::new(observer),
        }
    }

    pub async fn process(&mut self, buf: &[u8]) -> anyhow::Result<Vec<u8>> {
        let mut buf: Cow<'_, [u8]> = buf.into();
        let mut bytes = Vec::new();

        if !self.handshake_state {
            match self.handshake.process_bytes(&buf)? {
                InProgress {
                    response_bytes,
                } => return Ok(response_bytes),
                Completed {
                    // Any bytes that should be sent to the peer as a response.
                    response_bytes,
                    // Any bytes left over after completing the handshake.
                    remaining_bytes,
                } => {
                    bytes.extend_from_slice(&response_bytes);
                    buf = remaining_bytes.into();
                    self.handshake_state = true;
                },
            }
        }

        if self.handshake_state {
            let buf = self.session.process(&buf).await?;
            bytes.extend_from_slice(&buf);
        }

        Ok(bytes)
    }
}
