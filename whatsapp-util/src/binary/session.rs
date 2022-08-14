use crate::binary::codec;
use crate::binary::codec::{CodecInput, NodeCodec, TransposeOutput};
use crate::binary::node::Node;
use crate::model::Credentials;

pub use crate::Result;
pub use crate::model::session_store::SessionStore;

#[derive(Clone)]
pub struct Session {
    pub store: SessionStore,
    pub credentials: Credentials,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            store: SessionStore::default(),
            credentials: Credentials::default(),
        }
    }
}

impl Session {
    pub fn encode(&mut self, intro: bool, node: Node) -> Result<Vec<u8>> {
        let TransposeOutput::Encoded(encoded) = NodeCodec::transpose(
            &mut self.store,
            CodecInput::Encode(node)
        )? else { unreachable!() };

        codec::encode_frame(intro, &encoded)
    }

    pub fn encode_binary<T>(&self, intro: bool, input: T) -> Result<Vec<u8>>
    where
        T: AsRef<[u8]>
    {
        codec::encode_frame(intro, input)
    }

    pub fn decode_binary<T>(&self, input: T) -> Vec<Vec<u8>>
    where
        T: AsRef<[u8]>
    {
        codec::decode_frame(input)
    }

    pub fn decode<T>(&mut self, payload: T) -> Result<Vec<Node>>
    where
        T: AsRef<[u8]>
    {
        let decoded = codec::decode_frame(payload);
        let mut nodes = Vec::with_capacity(decoded.len());

        for segment in decoded {
            let TransposeOutput::Decoded(node) = NodeCodec::transpose(
                &mut self.store,
                CodecInput::Decode(&segment)
            )? else { unreachable!() };

            nodes.push(node);
        }

        Ok(nodes)
    }

}
