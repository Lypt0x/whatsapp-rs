use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait RequestModel {
    const ENDPOINT: &'static str;
    type Output;

    async fn deserialize(body: hyper::Body) -> Result<Self::Output>;
}
