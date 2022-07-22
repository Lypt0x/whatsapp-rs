use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to cipher: {0:?}")]
    AesCipherFail(aes_gcm::Error)
}