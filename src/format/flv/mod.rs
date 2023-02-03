mod format;

pub use format::*;
use bytes::*;

pub struct Flv {
    header: Header,
    header_state: bool,
    timestamp_cursor: u32,
    timestamp: u32,
    bytes: BytesMut,
}

impl Flv {
    pub fn new(header: Header) -> Self {
        Self {
            bytes: BytesMut::with_capacity(5000),
            header_state: false,
            timestamp_cursor: 0,
            timestamp: 0,
            header,
        }
    }

    pub fn encode(
        &mut self,
        frame: Frame,
        timestamp: u32,
        src: &[u8],
    ) -> usize {
        let mut size = 0;

        self.timestamp += if !(self.timestamp_cursor > timestamp
            || self.timestamp_cursor == 0)
        {
            timestamp - self.timestamp_cursor
        } else {
            0
        };

        if !self.header_state {
            size += self.header.encode(&mut self.bytes);
            self.header_state = true;
        }

        self.timestamp_cursor = timestamp;
        size + frame.encode(src, &mut self.bytes, self.timestamp)
    }

    pub fn flush_to(&mut self) -> Vec<u8> {
        let bytes = self.bytes[..].to_vec();
        self.bytes.clear();
        bytes
    }
}
