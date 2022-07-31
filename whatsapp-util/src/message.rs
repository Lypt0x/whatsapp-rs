use bytebuffer::ByteBuffer;

pub mod codec;

pub struct BinaryMessage {
    pub raw: ByteBuffer,
    pub decoded: Vec<Vec<u8>>,
}

impl BinaryMessage {
    pub fn new(raw: &[u8]) -> Self {
        let mut raw = ByteBuffer::from_bytes(raw);
        let mut decoded = vec![];

        while raw.len() - raw.get_rpos() >= 3 {
            let length = Self::decode_length(&mut raw);
            if length < 0 {
                continue;
            }

            decoded.push(raw.read_bytes(length as usize))
        }

        Self { raw, decoded }
    }

    pub fn decode_length(buffer: &mut ByteBuffer) -> i32 {
        (buffer.read_u8() as i32 & 0xFF << 16 as i32 | buffer.read_u16() as i32) as i32
    }
}
