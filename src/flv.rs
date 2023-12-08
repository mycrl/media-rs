use bytes::{BufMut, BytesMut};

#[repr(u8)]
#[allow(unused)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FlvHeader {
    Video = 0x01,
    Audio = 0x04,
    Full = 0x05,
}

impl FlvHeader {
    pub fn encode(&self, buf: &mut BytesMut) -> usize {
        buf.put_u8(0x46); // F
        buf.put_u8(0x4c); // L
        buf.put_u8(0x56); // V
        buf.put_u8(0x01); // version
        buf.put_u8(*self as u8); // type
        buf.put_u32(9); // size
        buf.put_u32(0);

        13
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FlvFrame {
    Audio = 0x08,
    Video = 0x09,
    Script = 0x12,
}

impl FlvFrame {
    pub fn encode(&self, src: &[u8], dst: &mut BytesMut, timestamp: u32) -> usize {
        dst.put_u8(*self as u8);
        dst.put_uint(src.len() as u64, 3);
        dst.put_u32(timestamp_xor(timestamp));
        dst.put_uint(0, 3);
        dst.put(src);
        dst.put_u32((src.len() + 11) as u32);

        15 + src.len()
    }
}

pub fn timestamp_xor(timestamp: u32) -> u32 {
    u32::from_be_bytes([
        ((timestamp >> 16) & 0xff) as u8,
        ((timestamp >> 8) & 0xff) as u8,
        (timestamp & 0xff) as u8,
        ((timestamp >> 24) & 0xff) as u8,
    ])
}

pub struct FlvEncoer {
    header: FlvHeader,
    header_state: bool,
    timestamp_cursor: u32,
    timestamp: u32,
    bytes: BytesMut,
}

impl FlvEncoer {
    pub fn new(header: FlvHeader) -> Self {
        Self {
            bytes: BytesMut::with_capacity(5000),
            header_state: false,
            timestamp_cursor: 0,
            timestamp: 0,
            header,
        }
    }

    pub fn encode(&mut self, frame: FlvFrame, timestamp: u32, src: &[u8]) -> usize {
        let mut size = 0;

        self.timestamp += if !(self.timestamp_cursor > timestamp || self.timestamp_cursor == 0) {
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
