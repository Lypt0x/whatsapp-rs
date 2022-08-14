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
pub(crate) use crate::util::*;
pub(crate) use super::node::*;

pub struct NodeCodec;

pub enum CodecInput<'a> {
    Encode(Node),
    Decode(&'a [u8])
}

pub enum TransposeOutput {
    Encoded(Vec<u8>),
    Decoded(Node),
}

impl NodeCodec {
    pub(crate) fn transpose(store: &mut SessionStore, codec: CodecInput) -> Result<TransposeOutput> {
        Ok(match codec {
            CodecInput::Encode(node) => TransposeOutput::Encoded({
                let encoded = NodeEncoder::encode(node)?;
                println!("Encoded: {:?}", encoded);

                store.encrypt(&encoded)?
            }),

            CodecInput::Decode(binary) => TransposeOutput::Decoded({
                let decrypted = store.decrypt(binary)?;

                NodeDecoder::decode(&decrypted)?
            })
        })
    }
}

pub(crate) fn encode_frame<T>(intro: bool, data: T) -> Result<Vec<u8>>
where
    T: AsRef<[u8]>
{
    println!("Encode intro: {}", intro);
    let mut buffer = ByteBuffer::from_bytes(if intro { &PROLOGUE } else { &[] });

    buffer.write_i32((data.as_ref().len() >> 16) as i32);
    buffer.write_i16((65535 & data.as_ref().len()) as i16);
    buffer.write_bytes(&data.as_ref());
    Ok(buffer.to_bytes())
}

pub(crate) fn decode_frame<T>(input: T) -> Vec<Vec<u8>>
where
    T: AsRef<[u8]>
{
    fn decode_length(buffer: &mut ByteBuffer) -> i32 {
        (buffer.read_u8() as i32 & 0xFF << 16 as i32 | buffer.read_u16() as i32) as i32
    }

    let mut raw = ByteBuffer::from_bytes(input.as_ref());
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
