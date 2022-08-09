pub mod client;

pub use whatsapp_rs_util::*;

#[cfg(test)]
mod tests {
    use crate::client::WebSocketClient;

    #[tokio::test]
    pub async fn test() {
        let mut client = WebSocketClient::default();
        client.connect().await.unwrap();
    }
}
