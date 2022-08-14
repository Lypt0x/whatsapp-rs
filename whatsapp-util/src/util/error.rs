use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to cipher: {0:?}")]
    AesCipherFail(aes_gcm::Error),

    #[error("Failed to transform: {0:?}")]
    IntoError(&'static str),
    
    #[error("The Stream has not been initialized yet")]
    StreamNotInitialized,

    #[error("The stream has already been initialized")]
    StreamAlreadyInitialized,

    #[error("The stream has already been called in a wrong state (ex. Hello! after Login)")]
    WrongState,
    
    // TODO: Automatic "reconnect"
    #[error("The device was missing during iq authentication, please redo the login")]
    IqMissingDevice,

    #[error("The signature during the iq authentication was invalid")]
    IqInvalidSignature,

    #[error("The message signature during the iq authentication was invalid")]
    IqInvalidMessageSignature,
    
    #[error("The node is not known by the protocol")]
    UnknownNode,

    #[error("Failed to connect to the WhatsApp WebSocket")]
    WebSocketConnectError,
    
    #[error("The stream received or transmitted an unexpected message")]
    UnexpectedMessage,

    #[error("Could not encode the input node: {0:?}")]
    EncodeNodeError(anyhow::Error),

    #[error("Could not encode the input binary: {0:?}")]
    EncodeBinaryError(anyhow::Error),
    
    #[error("The connection has been closed remotely")]
    WsClose
}
