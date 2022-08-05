use tokio_tungstenite::tungstenite::http::header::*;
use tokio_tungstenite::tungstenite::http::Request;
use whatsapp_rs_util::model::session::Session;

use crate::client::auth::AuthHandler;
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use whatsapp_rs_util::model::credentials::Credentials;
use whatsapp_rs_util::binary::handshake::Handshake;
use whatsapp_rs_util::binary;
use whatsapp_rs_util::binary::codec::NodeCodec;

use whatsapp_rs_util::protobuf::whatsapp::{ClientHello, MessageParser};

pub mod auth;

#[derive(Default, PartialEq)]
pub enum State {
    #[default]
    Handshake,

    Connected,
}

#[derive(Default)]
pub struct WebSocketClient {
    session: Option<Session>,
    credentials: Credentials,
    state: State,
}

impl WebSocketClient {
    pub fn init(&mut self, session: Session) {
        self.session = Some(session);
    }

    pub async fn connect(&mut self) -> Result<()> {
        if self.session.is_none() {
            self.session = Some(Session::default())
        }

        let (mut websocket, _) = tokio_tungstenite::connect_async(
            Request::builder()
                .uri("wss://web.whatsapp.com/ws/chat")
                .header(SEC_WEBSOCKET_KEY, "3zbjYJIgtLc2sjZJLvyK+Q==")
                .header(HOST, "web.whatsapp.com")
                .header(CONNECTION, "keep-alive, Upgrade")
                .header(UPGRADE, "websocket")
                .header(SEC_WEBSOCKET_VERSION, "13")
                .header(ORIGIN, "https://web.whatsapp.com")
                .body(())
                .unwrap(),
        )
        .await?;

        let mut hello = ClientHello::new();
        hello.ephemeral = self
            .credentials
            .ephemeral_keypair
            .public
            .to_bytes()
            .to_vec()
            .into();
        let hello_handshake = Handshake::create_hello_handshake(hello);
        let encoded_hello = binary::codec::encode_frame(true, &hello_handshake.write_to_bytes()?)?;
        websocket.send(Message::Binary(encoded_hello)).await?;

        while let Some(message) = websocket.next().await {
            match message.as_ref().expect("Failure at receive") {
                Message::Binary(payload) => {
                    if payload.as_slice() == [136, 2, 3, 243] {
                        println!("Server closed connection");
                        break;
                    }

                    let message = binary::codec::decode_frame(payload);
                    if message.is_empty() {
                        println!("Could not decode binary");
                        break;
                    }

                    if self.state == State::Handshake {
                        println!("Logging in..");

                        let decoded = message.first().expect("Decode failure");

                        let mut auth =
                            AuthHandler::new(self.session.as_mut().unwrap(), &mut self.credentials);

                        auth.login(decoded, &mut websocket).await?;

                        self.state = State::Connected;
                    } else {
                        println!("Connected, node");
                        let mut decoder = NodeCodec { session: self.session.as_mut().unwrap() };

                        for decoded in message {
                            decoder.decode(&decoded);
                        }

                        break;
                    }
                }
                _ => {
                    println!("Unknown binary received: {binary:?}")
                }
            }
        }

        Ok(())
    }
}
