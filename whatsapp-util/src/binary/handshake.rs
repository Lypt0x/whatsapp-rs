use crate::model::credentials::Credentials;
use crate::protobuf::whatsapp::client_payload::{
    ClientPayloadConnectReason, ClientPayloadConnectType,
};

use crate::protobuf::whatsapp::companion_props::CompanionPropsPlatformType;
use crate::protobuf::whatsapp::user_agent::{UserAgentPlatform, UserAgentReleaseChannel};
use crate::protobuf::whatsapp::web_info::WebInfoWebSubPlatform;
use crate::protobuf::whatsapp::{
    AppVersion, ClientFinish, ClientHello, ClientPayload, CompanionProps, CompanionRegData,
    HandshakeMessage, UserAgent, WebInfo,
};

use crate::security::keypair::Keypair;
use crate::security::{aes, hash, hkdf};

use protobuf::{EnumOrUnknown, Message, MessageField};
use crate::model::SessionStore;

pub use crate::Result;
use crate::security::AsNonce;
use super::{PROTOCOL, PROLOGUE};

pub struct Handshake<'a> {
    pub credentials: &'a mut Credentials,
    pub session: &'a mut SessionStore,

    hash: [u8; 32],
    crypto_key: [u8; 32],
    salt: [u8; 32],
    iv: u64,
}

impl<'a> Handshake<'a> {
    pub fn new(store: &'a mut SessionStore, credentials: &'a mut Credentials) -> Self {
        let auth = Self {
            credentials,
            session: store,
            hash: PROTOCOL,
            crypto_key: PROTOCOL,
            salt: PROTOCOL,
            iv: 0,
        };

        let public_key = auth.credentials.ephemeral_keypair.public.to_bytes();
        auth.rehash(&PROLOGUE).rehash(&public_key)
    }

    pub fn finish(&mut self) -> Result<()> {
        self.session.update(self.salt)
    }

    pub fn ephemeral_key(&'_ self) -> &'_ Keypair {
        &self.credentials.ephemeral_keypair
    }

    pub fn ephemeral_key_mut(&'_ mut self) -> &'_ mut Keypair {
        &mut self.credentials.ephemeral_keypair
    }

    pub fn noise_key(&'_ self) -> &'_ Keypair {
        &self.credentials.noise_keypair
    }

    pub fn noise_key_mut(&'_ mut self) -> &'_ mut Keypair {
        &mut self.credentials.noise_keypair
    }

    pub fn decrypt(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        let decrypted = aes::decrypt(
            self.crypto_key, self.hash,
            self.iv.get_increment_nonce_mut(), input
        )?;

        self.rehash_mut(input);
        Ok(decrypted)
    }

    pub fn encrypt(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        let decrypted = aes::encrypt(
            self.crypto_key, self.hash,
            self.iv.get_increment_nonce_mut(), input
        )?;

        self.rehash_mut(&decrypted);
        Ok(decrypted)
    }

    pub fn rehash(mut self, input: &[u8]) -> Self {
        self.hash = hash::sha256(&self.hash, input);
        self
    }

    pub fn rehash_mut(&mut self, input: &[u8]) -> &mut Self {
        self.hash = hash::sha256(&self.hash, input);
        self
    }

    pub fn mix(&mut self, input: &[u8]) {
        let expanded = hkdf::expand_extract(self.salt, input);
        self.salt = expanded.as_ref()[..32].try_into().unwrap();
        self.crypto_key = expanded.as_ref()[32..].try_into().unwrap();
        self.iv = 0;
    }

    pub fn create_finish_handshake(finish: ClientFinish) -> HandshakeMessage {
        HandshakeMessage {
            clientFinish: MessageField::some(finish),
            ..Default::default()
        }
    }

    pub fn create_hello_handshake(hello: ClientHello) -> HandshakeMessage {
        HandshakeMessage {
            clientHello: MessageField::some(hello),
            ..Default::default()
        }
    }

    // TODO: Make dis thing lil bit less hardcoded lol
    pub fn create_user_payload(&self) -> Result<ClientPayload> {
        let mut user_agent = UserAgent::new();
        let mut app_version = AppVersion::new();
        app_version.primary = 2.into();
        app_version.secondary = 2226.into();
        app_version.tertiary = 4.into();

        user_agent.platform = EnumOrUnknown::from(UserAgentPlatform::WEB).into();
        user_agent.appVersion = MessageField::some(app_version);
        user_agent.releaseChannel = EnumOrUnknown::from(UserAgentReleaseChannel::RELEASE).into();

        let mut web_info = WebInfo::new();
        web_info.webSubPlatform = EnumOrUnknown::from(WebInfoWebSubPlatform::WEB_BROWSER).into();

        let mut reg_data = CompanionRegData::new();
        let mut companion_props = CompanionProps::new();
        companion_props.os = String::from("whatsapprs").into();
        companion_props.platformType =
            EnumOrUnknown::from(CompanionPropsPlatformType::DESKTOP).into();
        companion_props.requireFullSync = false.into();

        reg_data.eRegid = self
            .credentials
            .registration_id
            .to_be_bytes()
            .to_vec()
            .into();
        reg_data.eKeytype = vec![5u8].into();
        reg_data.eIdent = self
            .credentials
            .identity_keypair
            .public
            .as_bytes()
            .to_vec()
            .into();
        reg_data.eSkeyId = self
            .credentials
            .signed_keypair
            .key_id
            .to_be_bytes()
            .to_vec()
            .into();
        reg_data.eSkeyVal = self
            .credentials
            .signed_keypair
            .key_pair
            .public
            .as_bytes()
            .to_vec()
            .into();
        reg_data.eSkeySig = self
            .credentials
            .signed_keypair
            .signature
            .as_ref()
            .to_vec()
            .into();
        reg_data.buildHash = hash::md5(b"2.2226.4").to_vec().into();
        reg_data.companionProps = companion_props.write_to_bytes()?.to_vec().into();

        let mut client_payload = ClientPayload::new();
        client_payload.passive = true.into();
        client_payload.userAgent = MessageField::some(user_agent);
        client_payload.webInfo = MessageField::some(web_info);
        client_payload.regData = MessageField::some(reg_data);
        client_payload.connectType =
            EnumOrUnknown::from(ClientPayloadConnectType::WIFI_UNKNOWN).into();
        client_payload.connectReason =
            EnumOrUnknown::from(ClientPayloadConnectReason::USER_ACTIVATED).into();
        Ok(client_payload)
    }
}
