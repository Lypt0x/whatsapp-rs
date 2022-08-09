use std::sync::Arc;
use futures::SinkExt;

use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;

pub use state::State;
use whatsapp_rs_util::binary::handshake::{Message as _, MessageField};
use whatsapp_rs_util::binary::node::Node;
use whatsapp_rs_util::model::Session;
use whatsapp_rs_util::protobuf::whatsapp::{ClientHello, HandshakeMessage};

use crate::client::auth::AuthHandler;
use crate::client::stream::processor::StreamProcessor;
pub use crate::Result;
pub use crate::util::error::Error;

pub mod state;
pub mod processor;

pub struct Stream {
	session: Arc<Mutex<Session>>,
	processor: StreamProcessor,
	ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
	state: State,
}

pub enum Transmission {
	Binary(Vec<u8>),
	Node(Node)
}

impl Stream {
	pub fn new(session: Option<Session>, ws: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
		let mut stream = Self {
			session: Arc::new(Mutex::new(session.unwrap_or_default())),
			processor: Default::default(),
			state: Default::default(),
			ws
		};

		stream.processor.session = Some(Arc::clone(&stream.session));
		stream
	}

	pub async fn send_hello(&mut self) -> Result<()> {
		let hello_handshake = HandshakeMessage {
			clientHello: MessageField::some(ClientHello {
				ephemeral: {
					let session = self.session.lock().await;
					session.credentials_ref().ephemeral_public().to_vec().into()
				},
				..Default::default()
			}),
			..Default::default()
		};

		self.send(
			Transmission::Binary(hello_handshake.write_to_bytes()?)
		).await
	}

	pub async fn login(&mut self, server_payload: &[u8]) -> Result<()> {
		let auth = AuthHandler::create_login({
			let session = self.session.lock().await;
			session
		}, server_payload).await?;

		self.send(Transmission::Binary(auth)).await
	}

	pub async fn process(&mut self, payload: &[u8]) -> Result<()> {
		match self.state {
			State::Hello | State::Handshake => {
				match self.processor.process_binary(payload).await {
					Ok(Some(server_hello)) => {
						self.state = State::Handshake;
						self.login(&server_hello).await
					},

					Ok(None) => {
						self.state = State::Iq;
						if let Some(processed) = self.processor.process_node(payload).await? {
							self.send(Transmission::Node(processed)).await?;
						}

						Ok(())
					},

					Err(error) => Err(error),
				}
			}

			State::Iq => {
				if let Some(processed) = self.processor.process_node(payload).await? {
					self.send(Transmission::Node(processed)).await?;
				}

				Ok(())
			}
		}
	}

	async fn send(&mut self, transmission: Transmission) -> Result<()> {
		let Self { session, ws, state, .. } = self;
		let mut session = session.lock().await;

		Ok(match transmission {
			Transmission::Binary(input) => {
				let encoded = session.encode_binary(state.is_default(), &input).map_err(Error::EncodeBinaryError)?;
				ws.send(Message::Binary(encoded)).await?
			}
			Transmission::Node(node) => {
				let encoded = session.encode(state.is_default(), node).map_err(Error::EncodeNodeError)?;
				ws.send(Message::Binary(encoded)).await?
			}
		})
	}

	pub fn ws(&mut self) -> &mut WebSocketStream<MaybeTlsStream<TcpStream>> {
		&mut self.ws
	}

	pub fn session(&self) -> Arc<Mutex<Session>> {
		Arc::clone(&self.session)
	}

}