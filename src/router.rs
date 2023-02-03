use crate::format::flv::*;
use bytes::Bytes;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
};

use tokio::sync::{
    RwLock,
    mpsc::*,
};

type A = SocketAddr;
type KeyFrame = HashMap<String, Vec<Payload>>;
type Senders = HashMap<String, HashMap<A, Sender<Payload>>>;

#[derive(Clone, Debug)]
pub struct Payload {
    timestamp: u32,
    bytes: Bytes,
    frame: Frame,
}

pub struct Router {
    senders: Arc<RwLock<Senders>>,
    keyframes: Arc<RwLock<KeyFrame>>,
    addrs: RwLock<HashMap<SocketAddr, String>>,
}

impl Router {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            addrs: RwLock::new(HashMap::with_capacity(1024)),
            keyframes: Arc::new(RwLock::new(HashMap::with_capacity(1024))),
            senders: Arc::new(RwLock::new(HashMap::with_capacity(1024))),
        })
    }

    pub async fn get_receiver(
        &self,
        addr: &SocketAddr,
        name: &str,
    ) -> Option<Reader> {
        let keyframes = self.keyframes.read().await;
        let keyframes = keyframes.get(name)?.as_slice();
        let (tx, rx) = channel(1);

        self.senders
            .write()
            .await
            .entry(name.to_string())
            .or_insert_with(HashMap::new)
            .insert(*addr, tx);
        self.addrs.write().await.insert(*addr, name.to_string());
        Some(Reader::new(keyframes, rx))
    }

    pub async fn get_sender(&self, addr: &SocketAddr, name: &str) -> Writer {
        self.addrs.write().await.insert(*addr, name.to_string());
        self.keyframes
            .write()
            .await
            .insert(name.to_string(), Vec::with_capacity(3));
        Writer::new(name, self.keyframes.clone(), self.senders.clone())
    }

    pub async fn remove(&self, addr: &SocketAddr) {
        if let Some(name) = self.addrs.write().await.remove(addr) {
            self.senders.write().await.remove(&name);
            self.keyframes.write().await.remove(&name);
        }
    }
}

#[derive(Default)]
pub struct WriterState {
    audio: bool,
    video: bool,
    metadata: bool,
}

pub struct Writer {
    failed_txs: Vec<SocketAddr>,
    keyframes: Arc<RwLock<KeyFrame>>,
    senders: Arc<RwLock<Senders>>,
    state: WriterState,
    name: String,
}

impl Writer {
    fn new(
        name: &str,
        keyframes: Arc<RwLock<KeyFrame>>,
        senders: Arc<RwLock<Senders>>,
    ) -> Self {
        Self {
            failed_txs: Vec::with_capacity(10),
            state: WriterState::default(),
            name: name.to_string(),
            keyframes,
            senders,
        }
    }

    pub async fn send(
        &mut self,
        frame: Frame,
        timestamp: u32,
        bytes: Bytes,
    ) -> Option<()> {
        let payload = Payload {
            timestamp,
            frame,
            bytes,
        };

        // The head of the flv needs media information frames and the audio and
        // video frames of the head, so here we check whether the information
        // has been written into the internal cache.
        let in_keyframe = match frame {
            Frame::Script if !self.state.metadata => {
                self.state.metadata = true;
                true
            },
            Frame::Video if !self.state.video => {
                self.state.video = true;
                true
            },
            Frame::Audio if !self.state.audio => {
                self.state.audio = true;
                true
            },
            _ => false,
        };

        // If these key frames are not passed to the channel, they are recorded
        // in the internal cache Can.
        if in_keyframe {
            self.keyframes
                .write()
                .await
                .get_mut(&self.name)?
                .push(payload);
        } else {
            {
                for (addr, sender) in
                    self.senders.read().await.get(&self.name)?
                {
                    if sender.send(payload.clone()).await.is_err() {
                        self.failed_txs.push(*addr);
                    }
                }
            }

            if !self.failed_txs.is_empty() {
                let mut senders = self.senders.write().await;
                let senders = senders.get_mut(&self.name)?;
                for addr in &self.failed_txs {
                    senders.remove(addr);
                }

                self.failed_txs.clear();
            }
        }

        Some(())
    }
}

pub struct Reader {
    inner: Receiver<Payload>,
    flv: Flv,
}

impl Reader {
    fn new(keyframes: &[Payload], inner: Receiver<Payload>) -> Self {
        let mut flv = Flv::new(Header::Full);
        for payload in keyframes {
            flv.encode(payload.frame, 0, &payload.bytes);
        }

        Self {
            inner,
            flv,
        }
    }

    pub async fn read(&mut self) -> Option<Vec<u8>> {
        let Payload {
            bytes,
            frame,
            timestamp,
        } = self.inner.recv().await?;
        self.flv.encode(frame, timestamp, &bytes);
        Some(self.flv.flush_to())
    }
}
