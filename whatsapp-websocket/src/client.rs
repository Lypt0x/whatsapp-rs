use tokio_tungstenite::tungstenite::http::header::*;
use tokio_tungstenite::tungstenite::http::Request;
use whatsapp_rs_util::handshake::session::Session;

use crate::client::auth::AuthHandler;
use anyhow::Result;
use futures::StreamExt;
use tokio_tungstenite::tungstenite::Message;
use whatsapp_rs_util::handshake::credentials::Credentials;
use whatsapp_rs_util::message::BinaryMessage;

pub mod auth;
pub mod codec;

#[derive(Default, Eq, PartialEq)]
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

        while let Some(message) = websocket.next().await {
            match message.as_ref().expect("Failure at receive") {
                Message::Binary(payload) => {
                    let message = BinaryMessage::new(payload);
                    if !message.decoded.is_empty() {
                        println!("Could not decode message");
                        break;
                    }

                    if self.state == State::default() {
                        println!("Logging in..");
                        let decoded = message.decoded.first().expect("Failure at codec stage");
                        let mut auth = AuthHandler::new(&mut self.credentials);
                        auth.login(decoded, &mut websocket).await?;

                        self.state = State::Connected;
                    } else {
                        println!("Connected: {:?}", message.decoded)
                    }
                }
                _ => {
                    println!("Unknown message received: {message:?}")
                }
            }
        }

        Ok(())
    }
}
