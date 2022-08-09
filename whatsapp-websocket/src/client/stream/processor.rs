use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use tokio::sync::Mutex;
use whatsapp_rs_util::binary::node::{DataExt, Node, Value};
use whatsapp_rs_util::model::{contact_jid, Session};
use whatsapp_rs_util::security;

use crate::Result;
use crate::util::error::Error;

#[derive(Default)]
pub struct StreamProcessor {
	pub session: Option<Arc<Mutex<Session>>>
}

impl StreamProcessor {

	pub async fn process_node(&mut self, input: &[u8]) -> Result<Option<Node>> {
		let nodes = self.session.as_ref().unwrap().lock().await.decode(input)?;

		for node in nodes {
			let result = self.digest(node).await?;
			if result.is_some() {
				return Ok(result)
			}
		}

		Ok(None)
	}

	pub async fn process_binary(&mut self, input: &[u8]) -> Result<Option<Vec<u8>>> {
		match input {
			// We usually receive this message when the server has decided to close this connection
			[0x88, 0x02, 0x03, 0xF3] => Err(anyhow!(Error::WsClose)),

			// Their message is a response of our hello identified by two zeros
			// So we're going to return a login require with the decoded frame
			[0, 0, ..] => Ok({
				let mut session = self.session.as_ref().unwrap().lock().await;
				session.decode_binary(input).swap_remove(0).into()
			}),

			// Their message is a node in an encoded frame
			// So we just return None to switch the next state to Iq and continue
			_ => Ok(None)
		}
	}

	async fn digest(&mut self, node: Node) -> Result<Option<Node>> {
		if node.id().unwrap() == "iq" {
			// TODO: error handling
			let container = node.children().first()
				.and_then(|child| child.as_array())
				.and_then(|nodes| nodes.first()).unwrap();

			let container: Node = container.try_into()?;

			if container.description() == "pair-device" {
				// Printing
				let reference = container.find_description("ref")
					.expect("Missing ref").content::<&str>().unwrap();

				let session = self.session.as_ref().unwrap().lock().await;
				let formatted_code = format!(
					"{},{noise_public},{identity_public},{companion}",
					reference,
					noise_public = security::base64::encode(session.credentials.noise_keypair.public.as_bytes()),
					identity_public = security::base64::encode(session.credentials.identity_keypair.public.as_bytes()),
					companion = security::base64::encode(session.credentials.signed_keypair.key_pair.public_key.public_key_bytes().unwrap())
				);

				// TODO: make greater
				qr2term::print_qr(formatted_code).unwrap();

				// Confirmation
				let request = Node::from_attributes(
					"iq".to_owned(),
					HashMap::from([
						("id".to_owned(), Value::String(node.attributes_clone()["id"].as_str().unwrap().to_string())),
						("type".to_owned(), Value::String("result".to_owned())),
						("to".to_owned(), Value::String(contact_jid::Server::Whatsapp.address().to_owned()))
					])
				);

				return Ok(Some(request))
			}
		}

		Ok(None)
	}

}