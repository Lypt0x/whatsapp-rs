use crate::client::request::RequestModel;
use async_trait::async_trait;
use hyper::Body;

use whatsapp_rs_util::protobuf::version::Version;

#[async_trait]
impl RequestModel for Version {
    const ENDPOINT: &'static str = "https://web.whatsapp.com/check-update?version=2.2212.7&platform=web";
    type Output = Self;

    async fn deserialize(body: Body) -> anyhow::Result<Self::Output> {
        let bytes = hyper::body::to_bytes(body).await?;
        let buf = std::str::from_utf8(&bytes[..])?;

        Ok(serde_json::from_str(buf)?)
    }
}
