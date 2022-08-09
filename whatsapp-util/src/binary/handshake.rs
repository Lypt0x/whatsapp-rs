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

use crate::security::{aes, hash, hkdf};

pub use protobuf::{EnumOrUnknown, Message, MessageField};
use crate::model::SessionStore;

pub use crate::Result;
use crate::security::AsNonce;
use super::{PROTOCOL, PROLOGUE};

pub struct Handshake {
    hash: [u8; 32],
    crypto_key: [u8; 32],
    salt: [u8; 32],
    iv: u64,
}

impl Handshake {
    pub fn new(ephemeral_public: [u8; 32]) -> Self {
        let auth = Self {
            hash: PROTOCOL,
            crypto_key: PROTOCOL,
            salt: PROTOCOL,
            iv: 0,
        };
        
        auth.rehash(&PROLOGUE).rehash(&ephemeral_public)
    }

    pub fn finish(&mut self, store: &mut SessionStore) -> Result<()> {
        store.update(self.salt)
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
    pub fn create_user_payload(credentials: &Credentials) -> Result<ClientPayload> {
        let mut user_agent = UserAgent::new();
        let mut app_version = AppVersion::new();
        app_version.primary = 2.into();
        app_version.secondary = 2228.into();
        app_version.tertiary = 14.into();

        user_agent.platform = EnumOrUnknown::from(UserAgentPlatform::WEB).into();
        user_agent.appVersion = MessageField::some(app_version);
        user_agent.releaseChannel = EnumOrUnknown::from(UserAgentReleaseChannel::RELEASE).into();

        let mut web_info = WebInfo::new();
        web_info.webSubPlatform = EnumOrUnknown::from(WebInfoWebSubPlatform::WEB_BROWSER).into();

        let mut reg_data = CompanionRegData::new();
        let mut companion_props = CompanionProps::new();
        companion_props.os = String::from("WhatsappWeb4j").into();
        companion_props.platformType =
            EnumOrUnknown::from(CompanionPropsPlatformType::DESKTOP).into();
        companion_props.requireFullSync = true.into();

        // id
        reg_data.eRegid = Self::int_to_bytes(credentials.registration_id as i32, 4).into();

        // keyType
        reg_data.eKeytype = Self::int_to_bytes(5, 1).into();

        // identifier
        reg_data.eIdent = credentials
            .identity_keypair
            .public
            .as_bytes()
            .to_vec()
            .into();

        // signatureId
        reg_data.eSkeyId = Self::int_to_bytes(credentials.registration_id as i32, 3).into();

        // signaturePublicKey
        reg_data.eSkeyVal = credentials
            .signed_keypair // signed instead
            .key_pair
            .public_key.public_key_bytes().unwrap()
            .to_vec()
            .into();

        // signature
        reg_data.eSkeySig = credentials
            .signed_keypair
            .signature
            .as_ref()
            .to_vec()
            .into();
        reg_data.buildHash = hash::md5(b"2.2228.14").to_vec().into();
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

    fn int_to_bytes(mut input: i32, length: usize) -> Vec<u8> {
        let mut result = vec![0; length];
        for i in (0..length).rev() {
            result[i] = (0xFF & input) as u8;
            input >>= 8;
        }

        result
    }
}
