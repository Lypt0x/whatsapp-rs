pub mod binary;
pub mod model;
pub mod protobuf;
pub mod security;
pub mod util;

pub use anyhow::Result;

#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use bytebuffer::ByteBuffer;
	use serde_json::Value;
	use crate::binary::codec::{NodeDecoder, NodeEncoder};
	use crate::binary::node::Node;

	#[test]
	pub fn encode_decode_node() {
		let input_node = Node::new(
			"action".to_owned(),
			HashMap::from([
				("key0".to_owned(), Value::String("Hello".to_owned())),
				("key1".to_owned(), Value::String("World".to_owned()))
			]),
			Value::String("Some content".to_owned())
		);

		let mut encoder = NodeEncoder::new(input_node.clone());
		let encoded = encoder.encode().unwrap();

		let mut decoder = NodeDecoder {
			buffer: ByteBuffer::from_bytes(&encoded)
		};

		assert_eq!(
			input_node, decoder.decode().unwrap()
		)
	}

}