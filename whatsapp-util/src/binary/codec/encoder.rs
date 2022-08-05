use super::*;

pub struct NodeEncoder {
	node: Node,
	buffer: ByteBuffer
}

impl NodeEncoder {
	pub fn new(node: Node) -> Self {
		Self {
			buffer: ByteBuffer::new(),
			node
		}
	}
	
	pub fn encode(&mut self) -> Vec<u8> {
		todo!()
	}
}