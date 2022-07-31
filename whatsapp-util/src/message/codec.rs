use crate::message::BinaryMessage;
use crate::protobuf::whatsapp::HandshakeMessage;
use anyhow::Result;
use bytebuffer::ByteBuffer;
use protobuf::Message;

pub const PROLOGUE: [u8; 4] = [87, 65, 5, 2];

pub fn encode(intro: bool, data: HandshakeMessage) -> Result<Vec<u8>> {
    let mut buffer = ByteBuffer::from_bytes(if intro { &PROLOGUE } else { &[] });

    let data = data.write_to_bytes()?;
    buffer.write_i32((data.len() >> 16) as i32);
    buffer.write_i16((65535 & data.len()) as i16);
    buffer.write_bytes(&data);
    Ok(buffer.to_bytes())
}

pub fn decode(input: &[u8]) -> Vec<Vec<u8>> {
    BinaryMessage::new(input).decoded
}
