mod iq;
mod error;
mod success;

use crate::Result;
use iq::*;
use whatsapp_rs_util::binary::node::{DataExt, Node};
use whatsapp_rs_util::model::Session;
use crate::stream::{Stream, Transmission};

pub struct DigestData {
	pub session: Session,
	pub node: Node
}

pub trait Digest {
	fn digest(node: DigestData) -> Result<Option<DigestData>>;
}

impl Stream<'_> {

	pub async fn digest(&mut self, node: Node) -> Result<()> {
		if node.id().is_none() {
			return Ok(());
		}

		let data = DigestData {
			session: self.client.session.clone(),
			node
		};

		if let Some(node) = match data.node.id().unwrap() {
			"iq" => <Iq as Digest>::digest(data)?,
			"success" => self.handle_success().await,
			"stream:error" => self.handle_error(data.node).await,
			"xmlstreamend" => None,

			_ => unimplemented!()
		} {
			let DigestData { session, node} = node;
			self.client.session = session;

			return self.client.send(Transmission::Node(node)).await
		}

		Ok(())
	}

}