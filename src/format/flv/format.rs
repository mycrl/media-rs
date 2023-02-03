use bytes::*;

#[repr(u8)]
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Header {
    Video = 0x01,
    Audio = 0x04,
    Full = 0x05,
}

impl Header {
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
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Frame {
    Audio = 0x08,
    Video = 0x09,
    Script = 0x12,
}

impl Frame {
    pub fn encode(
        &self,
        src: &[u8],
        dst: &mut BytesMut,
        timestamp: u32,
    ) -> usize {
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
