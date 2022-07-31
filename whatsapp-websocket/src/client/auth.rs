use std::fmt::Debug;
use whatsapp_rs_util::handshake::credentials::Credentials;
use whatsapp_rs_util::handshake::Handshake;

use anyhow::{anyhow, Result};
use futures::{Sink, SinkExt};
use tokio_tungstenite::tungstenite::Message;
use whatsapp_rs_util::handshake::session::Session;
use whatsapp_rs_util::message;

use whatsapp_rs_util::protobuf::whatsapp::MessageParser;
use whatsapp_rs_util::protobuf::whatsapp::{ClientFinish, HandshakeMessage};

pub struct AuthHandler<'a> {
    handshake: Handshake<'a>,
}

impl<'a> AuthHandler<'a> {
    pub fn new(session: &'a mut Session, credentials: &'a mut Credentials) -> Self {
        Self {
            handshake: Handshake::new(session, credentials),
        }
    }

    pub async fn login<S>(&mut self, payload: &[u8], sink: &mut S) -> Result<()>
    where
        S: SinkExt<Message> + Unpin,
        <S as Sink<Message>>::Error: Debug,
    {
        let handshake = HandshakeMessage::parse_from_bytes(payload)?;
        self.handshake.rehash_ref(handshake.serverHello.ephemeral());

        self.mix_ephemeral_shared(handshake.serverHello.ephemeral());
        self.mix_static(handshake.serverHello.static_())?;

        self.process_payload(handshake.serverHello.payload())?;

        let encrypted_key = self
            .handshake
            .encrypt(&self.handshake.noise_key().public.to_bytes())?;
        self.mix_noise_shared(handshake.serverHello.ephemeral());

        let user_payload = self.handshake.create_user_payload()?.write_to_bytes()?;
        let encrypted_payload = self.handshake.encrypt(&user_payload)?;

        let mut client_finish = ClientFinish::new();
        client_finish.static_ = encrypted_key.into();
        client_finish.payload = encrypted_payload.into();

        let handshake_request = Handshake::create_finish_handshake(client_finish);
        let encoded = message::codec::encode_frame(false, &handshake_request.write_to_bytes()?)?;

        self.handshake.finish();

        Ok(sink
            .send(Message::Binary(encoded))
            .await
            .map_err(|err| anyhow!("Sink error: {:?}", err))?)
    }

    fn mix_ephemeral_shared(&mut self, their_ephemeral: &[u8]) {
        let shared = self
            .handshake
            .ephemeral_key_mut()
            .exchange(their_ephemeral)
            .to_bytes();

        self.handshake.mix(&shared);
    }

    fn mix_noise_shared(&mut self, their_ephemeral: &[u8]) {
        let shared = self
            .handshake
            .noise_key_mut()
            .exchange(their_ephemeral)
            .to_bytes();
        self.handshake.mix(&shared);
    }

    fn mix_static(&mut self, their_static: &[u8]) -> Result<()> {
        let decoded_static = self.handshake.decrypt(their_static)?;

        let shared = self
            .handshake
            .ephemeral_key_mut()
            .exchange::<&[u8]>(decoded_static.as_slice().into())
            .to_bytes();
        self.handshake.mix(&shared);
        Ok(())
    }

    fn process_payload(&mut self, their_payload: &[u8]) -> Result<()> {
        let _ = self.handshake.decrypt(their_payload)?;
        Ok(())
    }
}
