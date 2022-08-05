pub mod decoder;
pub use decoder::*;

pub mod encoder;
pub use encoder::*;

pub use crate::Result;
use super::PROLOGUE;

pub(crate) use bytebuffer::ByteBuffer;
pub(crate) use std::collections::HashMap;
pub(crate) use serde_json::Value;
pub(crate) use anyhow::bail;

pub(crate) use crate::model::*;
pub(crate) use crate::security::*;
pub(crate) use crate::util::*;
pub(crate) use crate::security::AsNonce;
pub(crate) use super::node::*;

pub struct NodeCodec<'a> {
    pub session: &'a mut Session
}

// TODO: frame codec in front of node codec
impl<'a> NodeCodec<'a> {
    pub fn encode(&mut self, node: Node) -> Result<Vec<u8>> {
        let encoded = NodeEncoder::new(node).encode();

        aes::encrypt_no_hash(
            self.session.read_key,
            self.session.read_cnt.get_increment_nonce_mut(),
            &encoded,
        )
    }

    pub fn decode(&mut self, input: &[u8]) -> Result<Node> {
        let buffer = aes::decrypt_no_hash(
            self.session.read_key,
            self.session.read_cnt.get_increment_nonce_mut(),
            input,
        )?;

        NodeDecoder { buffer: ByteBuffer::from_bytes(&buffer) }.decode()
    }
}

pub fn encode_frame(intro: bool, data: &[u8]) -> Result<Vec<u8>> {
    let mut buffer = ByteBuffer::from_bytes(if intro { &PROLOGUE } else { &[] });

    buffer.write_i32((data.len() >> 16) as i32);
    buffer.write_i16((65535 & data.len()) as i16);
    buffer.write_bytes(&data);
    Ok(buffer.to_bytes())
}

pub fn decode_frame(input: &[u8]) -> Vec<Vec<u8>> {
    fn decode_length(buffer: &mut ByteBuffer) -> i32 {
        (buffer.read_u8() as i32 & 0xFF << 16 as i32 | buffer.read_u16() as i32) as i32
    }

    let mut raw = ByteBuffer::from_bytes(input);
    let mut decoded = vec![];

    while raw.len() - raw.get_rpos() >= 3 {
        let length = decode_length(&mut raw);
        if length < 0 {
            continue;
        }

        decoded.push(raw.read_bytes(length as usize))
    }

    decoded
}
