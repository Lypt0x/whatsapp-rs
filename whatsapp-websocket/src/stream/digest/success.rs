use whatsapp_rs_util::binary::node::Node;
use crate::stream::digest::DigestData;
use crate::stream::Stream;

impl Stream<'_> {
	pub async fn handle_success(&mut self) -> Option<DigestData> {
		self.client.query(
			"set",
			"passive",
			Node::from_attributes("active".to_owned(), [].into())
		).await.unwrap();

		// TODO: send pre keys when available
		todo!()
	}
}