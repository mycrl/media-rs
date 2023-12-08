use crate::flv::{FlvEncoer, FlvFrame, FlvHeader};

use std::{
    net::SocketAddr,
    sync::Arc,
    task::{Context, Poll},
};

use ahash::AHashMap;
use bytes::Bytes;
use tokio::sync::{mpsc::{channel, Receiver, Sender}, RwLock};

#[derive(Clone, Debug)]
pub struct Payload {
    timestamp: u32,
    bytes: Bytes,
    frame: FlvFrame,
}

#[derive(Default)]
pub struct Router {
    senders: Arc<RwLock<AHashMap<String, AHashMap<SocketAddr, Sender<Payload>>>>>,
    keyframes: Arc<RwLock<AHashMap<String, Vec<Payload>>>>,
    addrs: RwLock<AHashMap<SocketAddr, String>>,
}

impl Router {
    pub async fn get_receiver(&self, addr: &SocketAddr, name: &str) -> Option<RouterReceiver> {
        let keyframes = self.keyframes.read().await;
        let keyframes = keyframes.get(name)?.as_slice();
        let (tx, rx) = channel(1);

        self.senders
            .write()
            .await
            .entry(name.to_string())
            .or_insert_with(AHashMap::new)
            .insert(*addr, tx);
        self.addrs.write().await.insert(*addr, name.to_string());
        Some(RouterReceiver::new(keyframes, rx))
    }

    pub async fn get_sender(&self, addr: &SocketAddr, name: &str) -> RouterSender {
        self.addrs.write().await.insert(*addr, name.to_string());
        self.keyframes
            .write()
            .await
            .insert(name.to_string(), Vec::with_capacity(3));
        RouterSender::new(name, self.keyframes.clone(), self.senders.clone())
    }

    pub async fn remove(&self, addr: &SocketAddr) {
        if let Some(name) = self.addrs.write().await.remove(addr) {
            self.senders.write().await.remove(&name);
            self.keyframes.write().await.remove(&name);
        }
    }
}

#[derive(Default)]
pub struct RouterSenderState {
    audio: bool,
    video: bool,
    metadata: bool,
}

impl RouterSenderState {
    pub fn in_keyframe(&mut self, frame: FlvFrame) -> bool {
        // The head of the flv needs media information frames and the audio and
        // video frames of the head, so here we check whether the information
        // has been written into the internal cache.
        match frame {
            FlvFrame::Script if !self.metadata => {
                self.metadata = true;
                true
            }
            FlvFrame::Video if !self.video => {
                self.video = true;
                true
            }
            FlvFrame::Audio if !self.audio => {
                self.audio = true;
                true
            }
            _ => false,
        }
    }
}

pub struct RouterSender {
    failed_txs: Vec<SocketAddr>,
    keyframes: Arc<RwLock<AHashMap<String, Vec<Payload>>>>,
    senders: Arc<RwLock<AHashMap<String, AHashMap<SocketAddr, Sender<Payload>>>>>,
    state: RouterSenderState,
    name: String,
}

impl RouterSender {
    fn new(
        name: &str,
        keyframes: Arc<RwLock<AHashMap<String, Vec<Payload>>>>,
        senders: Arc<RwLock<AHashMap<String, AHashMap<SocketAddr, Sender<Payload>>>>>,
    ) -> Self {
        Self {
            failed_txs: Vec::with_capacity(10),
            state: RouterSenderState::default(),
            name: name.to_string(),
            keyframes,
            senders,
        }
    }

    pub async fn send(&mut self, frame: FlvFrame, timestamp: u32, bytes: Bytes) -> Option<()> {
        let payload = Payload {
            timestamp,
            frame,
            bytes,
        };

        // If these key frames are not passed to the channel, they are recorded
        // in the internal cache Can.
        if self.state.in_keyframe(frame) {
            self.keyframes
                .write()
                .await
                .get_mut(&self.name)?
                .push(payload);
        } else {
            {
                for (addr, sender) in self.senders.read().await.get(&self.name)? {
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

pub struct RouterReceiver {
    receiver: Receiver<Payload>,
    encoder: FlvEncoer,
}

impl RouterReceiver {
    fn new(keyframes: &[Payload], receiver: Receiver<Payload>) -> Self {
        let mut encoder = FlvEncoer::new(FlvHeader::Full);
        for payload in keyframes {
            encoder.encode(payload.frame, 0, &payload.bytes);
        }

        Self { receiver, encoder }
    }

    pub async fn read(&mut self) -> Option<Vec<u8>> {
        let Payload {
            bytes,
            frame,
            timestamp,
        } = self.receiver.recv().await?;
        self.encoder.encode(frame, timestamp, &bytes);
        Some(self.encoder.flush_to())
    }

    pub fn poll_read(&mut self, cx: &mut Context<'_>) -> Poll<Option<Vec<u8>>> {
        let Payload {
            bytes,
            frame,
            timestamp,
        } = match self.receiver.poll_recv(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(res) => match res {
                Some(payload) => payload,
                None => return Poll::Ready(None),
            },
        };

        self.encoder.encode(frame, timestamp, &bytes);
        Poll::Ready(Some(self.encoder.flush_to()))
    }
}
