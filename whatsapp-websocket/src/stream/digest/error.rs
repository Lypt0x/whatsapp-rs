use whatsapp_rs_util::binary::node::{DataExt, Node};
use crate::stream::digest::DigestData;
use crate::stream::Stream;

pub enum StreamError {
	ForceReconnect,
	Unauthorized
}

impl From<u32> for StreamError {
	fn from(input: u32) -> Self {
		match input {
			515 => Self::ForceReconnect,
			401 => Self::Unauthorized,
			_ => unimplemented!()
		}
	}
}

impl Stream<'_> {
	pub async fn handle_error(&mut self, node: Node) -> Option<DigestData> {
		let error: StreamError = node.error_code().expect("Expected error code").into();
		match error {
			StreamError::ForceReconnect => self.client.close(true).await,
			StreamError::Unauthorized => todo!(),
		}

		None
	}
}