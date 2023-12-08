mod message;

use super::RtmpObserver;

use anyhow::Result;
use bytes::Bytes;
use message::Msg;
use rml_rtmp::rml_amf0::{serialize, Amf0Value};
use rml_rtmp::{chunk_io::*, messages::MessagePayload, messages::*, time::RtmpTimestamp};

pub struct Command {
    encoder: ChunkSerializer,
}

impl Command {
    pub fn new() -> Self {
        Self {
            encoder: ChunkSerializer::new(),
        }
    }

    fn encode(&mut self, id: u32, msgs: Vec<RtmpMessage>) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        let timestamp = RtmpTimestamp { value: 0 };
        for msg in msgs {
            let payload = msg.into_message_payload(timestamp, id)?;
            let bytes = self.encoder.serialize(&payload, false, false)?.bytes;
            buf.extend_from_slice(&bytes);
        }

        Ok(buf)
    }

    pub fn set_max_chunk_size(&mut self, size: u32) -> Result<Vec<u8>> {
        let timestamp = RtmpTimestamp { value: 0 };
        Ok(self.encoder.set_max_chunk_size(size, timestamp)?.bytes)
    }

    pub fn connect(&mut self, id: u32) -> Result<Vec<u8>> {
        self.encode(
            id,
            vec![
                Msg::WindowAcknowledgement.into(),
                Msg::SetPeerBandwidth.into(),
                Msg::ConnectSuccess.into(),
            ],
        )
    }

    pub fn publish(&mut self, id: u32) -> Result<Vec<u8>> {
        self.encode(id, vec![Msg::PublishSuccess.into()])
    }

    pub fn create_stream(&mut self, id: u32) -> Result<Vec<u8>> {
        self.encode(id, vec![Msg::CreateSreamSuccess.into()])
    }
}

pub struct Session {
    app: Option<String>,
    key: Option<String>,
    decoder: ChunkDeserializer,
    observer: Box<dyn RtmpObserver>,
    command: Command,
}

impl Session {
    pub fn new<T>(observer: T) -> Self
    where
        T: RtmpObserver + 'static,
    {
        Self {
            app: None,
            key: None,
            observer: Box::new(observer),
            decoder: ChunkDeserializer::new(),
            command: Command::new(),
        }
    }

    pub fn set_max_chunk_size(&mut self, size: u32) -> Result<Vec<u8>> {
        self.decoder.set_max_chunk_size(size as usize)?;
        self.command.set_max_chunk_size(size)
    }

    pub async fn amf_value_command(
        &mut self,
        id: u32,
        name: &str,
        args: Vec<Amf0Value>,
        obj: Amf0Value,
    ) -> Result<Option<Vec<u8>>> {
        Ok(match name {
            "createStream" => Some(self.command.create_stream(id)?),
            "publish" => Some(self.command.publish(id)?),
            "connect" => {
                if let Amf0Value::Object(info) = obj {
                    if let Some(Amf0Value::Utf8String(app)) = info.get("app") {
                        let _ = self.app.insert(app.to_string());
                    }
                }

                Some(self.command.connect(id)?)
            }
            "releaseStream" => {
                if let Some(Amf0Value::Utf8String(key)) = args.get(0) {
                    let _ = self.key.insert(key.to_string());
                    if let (Some(a), Some(k)) = (&self.app, &self.key) {
                        self.observer.guard(a, k).await;
                    }
                }

                None
            }
            _ => None,
        })
    }

    pub async fn payload(&mut self, payload: MessagePayload) -> Result<Option<Vec<u8>>> {
        Ok(match payload.to_rtmp_message()? {
            RtmpMessage::Amf0Command {
                additional_arguments: args,
                command_object: obj,
                command_name,
                ..
            } => {
                self.amf_value_command(payload.message_stream_id, &command_name, args, obj)
                    .await?
            }
            RtmpMessage::AudioData { data } => {
                self.observer
                    .audio_data(payload.timestamp.value, data)
                    .await;
                None
            }
            RtmpMessage::VideoData { data } => {
                self.observer
                    .video_data(payload.timestamp.value, data)
                    .await;
                None
            }
            RtmpMessage::SetChunkSize { size } => Some(self.set_max_chunk_size(size)?),
            RtmpMessage::Amf0Data { values } => {
                if let Some(Amf0Value::Utf8String(key)) = values.get(0) {
                    if key.as_str() == "@setDataFrame" {
                        let bytes = serialize(&values)?;
                        let bytes = Bytes::copy_from_slice(&bytes[16..]);
                        self.observer.data_frame(bytes).await;
                    }
                }

                None
            }
            _ => None,
        })
    }

    pub async fn process(&mut self, buf: &[u8]) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let mut is_first = true;

        loop {
            // It is expected that consumers will call get_next_message() in a
            // loop until None is returned. Since it is important not to keep
            // sending it the same bytes over and over again an empty slice must
            // be passed in for subsequent calls.
            let buf = if is_first {
                is_first = false;
                buf
            } else {
                &[]
            };

            if let Some(p) = self.decoder.get_next_message(buf)? {
                if let Some(buf) = self.payload(p).await? {
                    bytes.extend_from_slice(&buf);
                }
            } else {
                break;
            }
        }

        Ok(bytes)
    }
}
