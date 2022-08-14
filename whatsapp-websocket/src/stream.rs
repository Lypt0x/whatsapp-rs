
pub mod processor;
pub mod digest;

use anyhow::bail;

pub use whatsapp_rs_util::binary::state::State;
use whatsapp_rs_util::binary::handshake::{Message as _, MessageField};
use whatsapp_rs_util::binary::node::Node;
use whatsapp_rs_util::protobuf::whatsapp::{ClientHello, HandshakeMessage};

use crate::client::auth::AuthHandler;
use crate::client::WebSocketClient;
pub use crate::Result;
pub use crate::util::error::Error;

pub struct Stream<'a> {
	client: &'a mut WebSocketClient,
}

pub enum Transmission {
	Binary(Vec<u8>),
	Node(Node)
}

impl<'a> Stream<'a> {
	pub async fn new(client: &'a mut WebSocketClient) -> Result<Stream<'a>> {
		let mut stream = Self {
			client
		};

		stream.send_hello().await?;
		Ok(stream)
	}

	pub async fn send_hello(&mut self) -> Result<()> {
		let state = self.client.state;

		if state.is_default() {
			let hello_handshake = HandshakeMessage {
				clientHello: MessageField::some(ClientHello {
					ephemeral: self.client.session.credentials.ephemeral_public().to_vec().into(),
					..Default::default()
				}),
				..Default::default()
			};

			let result = self.client.send(
				Transmission::Binary(hello_handshake.write_to_bytes()?)
			).await;
			self.client.state = State::Handshake;

			result
		} else {
			bail!(Error::WrongState)
		}
	}

	pub async fn login<T>(&mut self, server_payload: T) -> Result<()>
	where
		T: AsRef<[u8]>
	{
		let auth = AuthHandler::create_login({
			&mut self.client.session
		}, server_payload.as_ref()).await?;

		let result = self.client.send(Transmission::Binary(auth)).await;
		self.client.state = State::Connected;

		result
	}

}