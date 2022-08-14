#![feature(let_else)]

extern crate core;

pub mod client;
pub mod stream;

use tokio_tungstenite::tungstenite::http::header::{CONNECTION, HOST, ORIGIN, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION, UPGRADE};
use tokio_tungstenite::tungstenite::http::Request;
pub use whatsapp_rs_util::*;
pub use crate::Result;

#[cfg(test)]
mod tests {
    use crate::client::WebSocketClient;

    #[tokio::test]
    pub async fn test() {
        // Probably not the best way to test this. This is fine for now.
        let mut client = WebSocketClient::new(None);
        client.connect().await.unwrap();
    }
}

pub fn form_ws_request() -> Result<Request<()>> {
    Request::builder()
        .uri("wss://web.whatsapp.com/ws/chat")
        .header(SEC_WEBSOCKET_KEY, "3zbjYJIgtLc2sjZJLvyK+Q==")
        .header(HOST, "web.whatsapp.com")
        .header(CONNECTION, "keep-alive, Upgrade")
        .header(UPGRADE, "websocket")
        .header(SEC_WEBSOCKET_VERSION, "13")
        .header(ORIGIN, "https://web.whatsapp.com")
        .body(()).map_err(Into::into)
}