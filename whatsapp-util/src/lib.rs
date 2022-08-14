#![feature(let_else)]

pub mod binary;
pub mod model;
pub mod protobuf;
pub mod security;
pub mod util;

pub use anyhow::Result;

#[cfg(test)]
mod tests {
	use serde_json::Value;
	use crate::binary::codec::{NodeDecoder, NodeEncoder};
	use crate::binary::node::Node;

	#[test]
	pub fn encode_decode_node() {
		let node = Node::new(
			"iq".to_owned(),
			[
				// TODO: I definitely need to make it more generic
				("Hello".to_owned(), "100".into()),
				("To".to_owned(), "whatsapp-rs".to_owned().into())
			].into(),
			Value::String("This is some content".to_owned())
		);

		let encoded = NodeEncoder::encode(node.clone()).unwrap();
		let decoded = NodeDecoder::decode(encoded.as_slice()).unwrap();

		assert_eq!(
			node,
			decoded
		);
	}

	#[test]
	pub fn encode_decode_recursive_nodes() {
		let node = Node::new(
			"iq".to_owned(),
			[
				("Hello".to_owned(), "100".into()),
				("To".to_owned(), "whatsapp-rs".to_owned().into())
			].into(),
			Value::Array(vec![Node::serialize(Node::new(
				"pair".to_owned(),
				[
					("So".to_owned(), "23493274".into()),
					("is".to_owned(), "a number".to_owned().into())
				].into(),
				Value::String("This is definitely some content".to_owned())
			)).unwrap()])
		);

		let encoded = NodeEncoder::encode(node.clone()).unwrap();
		let decoded = NodeDecoder::decode(encoded.as_slice()).unwrap();

		assert_eq!(
			node,
			decoded
		);
	}

}