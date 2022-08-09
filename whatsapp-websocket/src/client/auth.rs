use whatsapp_rs_util::binary::handshake::Handshake;

use anyhow::Result;
use tokio::sync::MutexGuard;
use whatsapp_rs_util::model::Session;

use whatsapp_rs_util::protobuf::whatsapp::MessageParser;
use whatsapp_rs_util::protobuf::whatsapp::{ClientFinish, HandshakeMessage};
use whatsapp_rs_util::security::keypair::Keypair;

pub struct AuthHandler;

impl<'a> AuthHandler {
    pub async fn create_login(
        mut session: MutexGuard<'_, Session>,
        payload: &[u8],
    ) -> Result<Vec<u8>> {
        let mut handshake_auth = Handshake::new(session.credentials.ephemeral_public());
        let credentials = &mut session.credentials;

        let handshake = HandshakeMessage::parse_from_bytes(payload)?;
        handshake_auth.rehash_mut(handshake.serverHello.ephemeral());


        Self::mix_ephemeral_shared(
            &mut handshake_auth,
            &mut credentials.ephemeral_keypair,
            handshake.serverHello.ephemeral()
        );

        Self::mix_static(
            &mut handshake_auth,
            &mut credentials.ephemeral_keypair,
            handshake.serverHello.static_()
        )?;

        Self::process_payload(
            &mut handshake_auth,
            handshake.serverHello.payload()
        )?;

        let encrypted_key = handshake_auth
            .encrypt(&credentials.noise_keypair.public.to_bytes())?;
        Self::mix_noise_shared(
            &mut handshake_auth,
            &mut credentials.noise_keypair,
            handshake.serverHello.ephemeral()
        );

        let user_payload = Handshake::create_user_payload(credentials)?.write_to_bytes()?;
        let encrypted_payload = handshake_auth.encrypt(&user_payload)?;

        let mut client_finish = ClientFinish::new();
        client_finish.static_ = encrypted_key.into();
        client_finish.payload = encrypted_payload.into();

        let handshake_request = Handshake::create_finish_handshake(client_finish);

        handshake_auth.finish(&mut session.store)?;
        Ok(handshake_request.write_to_bytes()?)
    }

    fn mix_ephemeral_shared(handshake: &mut Handshake, ephemeral: &mut Keypair, their_ephemeral: &[u8]) {
        let shared = ephemeral
            .exchange(their_ephemeral)
            .to_bytes();

        handshake.mix(&shared);
    }

    fn mix_noise_shared(handshake: &mut Handshake, noise: &mut Keypair, their_ephemeral: &[u8]) {
        let shared = noise
            .exchange(their_ephemeral)
            .to_bytes();
        handshake.mix(&shared);
    }

    fn mix_static(handshake: &mut Handshake, ephemeral: &mut Keypair, their_static: &[u8]) -> Result<()> {
        let decoded_static = handshake.decrypt(their_static)?;

        let shared = ephemeral
            .exchange::<&[u8]>(decoded_static.as_slice().into())
            .to_bytes();
        handshake.mix(&shared);
        Ok(())
    }

    fn process_payload(handshake: &mut Handshake, their_payload: &[u8]) -> Result<()> {
        let _ = handshake.decrypt(their_payload)?;
        Ok(())
    }
}
