use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to cipher: {0:?}")]
    AesCipherFail(aes_gcm::Error),

    #[error("Failed to transform: {0:?}")]
    IntoError(&'static str),
    
    #[error("The Stream has not been initialized yet")]
    StreamNotInitialized,
    
    #[error("Could not encode the input node: {0:?}")]
    EncodeNodeError(anyhow::Error),

    #[error("Could not encode the input binary: {0:?}")]
    EncodeBinaryError(anyhow::Error),
    
    #[error("The connection has been closed remotely")]
    WsClose
}
