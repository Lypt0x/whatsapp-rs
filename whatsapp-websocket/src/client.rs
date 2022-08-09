use tokio_tungstenite::tungstenite::http::header::*;
use tokio_tungstenite::tungstenite::http::Request;

use anyhow::Result;
use tokio_tungstenite::tungstenite::Message;
use futures::StreamExt;

use crate::client::stream::Stream;

pub mod auth;
pub mod stream;

#[derive(Default)]
pub struct WebSocketClient {
    stream: Option<Stream>,
}

impl WebSocketClient {
    pub async fn connect(&mut self) -> Result<()> {
        let (ws, _) = tokio_tungstenite::connect_async(
            Request::builder()
                .uri("wss://web.whatsapp.com/ws/chat")
                .header(SEC_WEBSOCKET_KEY, "3zbjYJIgtLc2sjZJLvyK+Q==")
                .header(HOST, "web.whatsapp.com")
                .header(CONNECTION, "keep-alive, Upgrade")
                .header(UPGRADE, "websocket")
                .header(SEC_WEBSOCKET_VERSION, "13")
                .header(ORIGIN, "https://web.whatsapp.com")
                .body(())?,
        ).await?;
        
        self.stream = Stream::new(None, ws).into();
        let stream = self.stream.as_mut().unwrap();

        stream.send_hello().await?;

        Ok(while let Some(message) = stream.ws().next().await {
            match message.as_ref().expect("Failure at receive") {
                Message::Binary(payload) => {
                    stream.process(&payload).await?
                }
                _ => {
                    println!("Unknown binary received")
                }
            }
        })
    }
}
