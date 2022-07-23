pub mod request;
pub mod version;

use hyper_tls::HttpsConnector;
use hyper::{Body, Client as Http, Uri};
use hyper::client::HttpConnector;
use crate::client::request::RequestModel;
use anyhow::Result;

pub struct Client {
    http: Http<HttpsConnector<HttpConnector>, Body>
}

impl Client {

    pub async fn request<T: RequestModel>(&self) -> Result<T::Output> {
        let request = self.http.get(Uri::from_static(T::ENDPOINT)).await?;
        T::deserialize(request.into_body()).await
    }

}

impl Default for Client {
    fn default() -> Self {
        let http = Http::builder()
            .build(HttpsConnector::new());

        Self { http }
    }
}