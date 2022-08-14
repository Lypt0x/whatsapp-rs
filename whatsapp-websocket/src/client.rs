pub mod auth;

use std::collections::HashMap;
use anyhow::{bail, Result};
use futures::{SinkExt, StreamExt};
use futures::stream::SplitSink;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use whatsapp_rs_util::binary::node::{Node, Value};
use whatsapp_rs_util::binary::state::State;
use whatsapp_rs_util::model::{Server, Session};
use whatsapp_rs_util::security::Error;
use crate::stream::{Stream, Transmission};

pub struct WebSocketClient {
    sink: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    pub session: Session,
    pub state: State,
}

impl WebSocketClient {

    pub fn new(session: Option<Session>) -> Self {
        // This will be important when we want to restore the old key exchange
        Self { session: session.unwrap_or_default(), sink: None, state: State::default() }
    }

    pub async fn connect(&mut self) -> Result<()> {
        if !self.state.is_default() && self.state != State::Closed { bail!(Error::StreamAlreadyInitialized) }

        // I think this could be more consistent actually lol
        while self.state == State::Reconnect || self.state.is_default() {
            self.state = State::default();

            let (websocket, response) = tokio_tungstenite::connect_async(crate::form_ws_request()?).await?;
            if !response.status().is_informational() { bail!(Error::WebSocketConnectError) }

            let (tx, mut rx) = websocket.split();
            self.sink = tx.into();

            let mut stream = Stream::new(self).await?;
            while let Some(frame) = rx.next().await {
                match frame {
                    Ok(Message::Binary(message)) => {
                        stream.process(message).await?
                    },

                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub async fn close(&mut self, reconnect: bool) {
        if let Some(mut sink) = self.sink.take() {
            self.state = if reconnect { State::Reconnect } else { State::Closed };
            self.session.store.encode_key = [0u8; 32];
            self.session.store.decode_key = [0u8; 32];
            self.session.store.read_cnt = 0;
            self.session.store.write_cnt = 0;

            sink.send(Message::Close(None)).await.unwrap();
        }
    }

    pub(crate) async fn send(&mut self, transmission: Transmission) -> Result<()> {
        let sink = self.sink.as_mut().ok_or(Error::StreamNotInitialized)?;

        let state = self.state;
        let session = &mut self.session;

        Ok(match transmission {
            Transmission::Binary(input) => {
                let encoded = session.encode_binary(state.is_default(), &input)
                    .map_err(Error::EncodeBinaryError)?;

                sink.send(Message::Binary(encoded)).await?
            }

            Transmission::Node(node) => {
                let encoded = session.encode(state.is_default(), node)
                    .map_err(Error::EncodeNodeError)?;

                sink.send(Message::Binary(encoded)).await?
            }
        })
    }

    pub(crate) async fn query(&mut self, method: &str, category: &str, body: Node) -> Result<()> {
        // TODO: Query builder
        let attributes = HashMap::from([
            ("id".into(), Value::Null),
            ("type".into(), method.into()),
            ("to".into(), Server::Whatsapp.address().into()),
            ("xmlns".into(), category.into())
        ]);

        self.send(Transmission::Node(
            Node::new("iq".to_owned(), attributes, Node::serialize(body).unwrap())
        )).await
    }

}
