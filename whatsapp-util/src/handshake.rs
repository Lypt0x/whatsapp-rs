pub mod credentials;
pub mod session;

pub const PROTOCOL: [u8; 32] = *b"Noise_XX_25519_AESGCM_SHA256\0\0\0\0";
pub const PROLOGUE: [u8; 4] = [87, 65, 5, 2];

use crate::handshake::credentials::Credentials;
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
pub use crate::security::keypair::Keypair;
use crate::security::{aes, hash, hkdf};
pub use anyhow::Result;

use protobuf::{EnumOrUnknown, Message, MessageField};

pub struct Handshake<'a> {
    pub credentials: &'a mut Credentials,

    hash: [u8; 32],
    crypto_key: [u8; 32],
    salt: [u8; 32],
    iv: u64,
}

impl<'a> Handshake<'a> {
    pub fn new(credentials: &'a mut Credentials) -> Self {
        let auth = Self {
            credentials,
            hash: PROTOCOL,
            crypto_key: PROTOCOL,
            salt: PROTOCOL,
            iv: 0,
        };

        let public_key = auth.credentials.ephemeral_keypair.public.to_bytes();
        auth.rehash(&PROLOGUE).rehash(&public_key)
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
        let decrypted = aes::decrypt(self.crypto_key, self.hash, self.iv_as_nonce(), input)?;
        self.iv += 1;

        self.rehash_ref(input);
        Ok(decrypted)
    }

    pub fn encrypt(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        let decrypted = aes::encrypt(self.crypto_key, self.hash, self.iv_as_nonce(), input)?;
        self.iv += 1;

        self.rehash_ref(&decrypted);
        Ok(decrypted)
    }

    fn iv_as_nonce(&self) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        let src: [u8; 8] = self.iv.to_be_bytes();

        nonce[4..].copy_from_slice(&src);
        nonce
    }

    pub fn rehash(mut self, input: &[u8]) -> Self {
        self.hash = hash::sha256(&self.hash, input);
        self
    }

    pub fn rehash_ref(&mut self, input: &[u8]) -> &mut Self {
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
