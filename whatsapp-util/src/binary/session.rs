use crate::binary::codec;
use crate::binary::codec::NodeCodec;
use crate::binary::node::Node;
use crate::model::Credentials;

pub use crate::Result;
pub use crate::model::session_store::SessionStore;

pub struct Session {
    pub store: SessionStore,
    pub credentials: Credentials
}

impl Default for Session {
    fn default() -> Self {
        Self {
            store: SessionStore::default(),
            credentials: Credentials::default()
        }
    }
}

impl Session {
    pub fn encode(&mut self, intro: bool, node: Node) -> Result<Vec<u8>> {
        let encoded = NodeCodec::encode(&mut self.store, node)?;
        codec::encode_frame(intro, &encoded)
    }

    pub fn encode_binary(&mut self, intro: bool, input: &[u8]) -> Result<Vec<u8>> {
        codec::encode_frame(intro, input)
    }

    pub fn decode_binary(&mut self, input: &[u8]) -> Vec<Vec<u8>> {
        codec::decode_frame(input)
    }

    pub fn decode(&mut self, payload: &[u8]) -> Result<Vec<Node>> {
        let decoded = codec::decode_frame(payload);
        let mut nodes = Vec::with_capacity(decoded.len());

        for segment in decoded {
            nodes.push(NodeCodec::decode(&mut self.store, &segment)?);
        }

        Ok(nodes)
    }
    
    pub fn credentials_ref(&self) -> &Credentials {
        &self.credentials
    }

}
