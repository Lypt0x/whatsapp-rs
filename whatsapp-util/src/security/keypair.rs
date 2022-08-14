pub use ed25519_dalek::{Keypair as EdKeypair, Signature};
use libsignal_protocol::{KeyPair, PrivateKey};
use rand_core::OsRng;
pub use x25519_dalek::{PublicKey, SharedSecret};
use x25519_dalek::StaticSecret;
use crate::Result;

// For now, we mix some other crates that works with curve25519 as well
#[derive(Clone)]
pub struct Keypair {
    pub public: PublicKey,
    pub secret: StaticSecret,
}

#[derive(Clone)]
pub struct SignedKeypair {
    pub key_pair: KeyPair,
    pub signature: Box<[u8]>,
    pub key_id: i32,
}

impl SignedKeypair {
    pub fn new(identity_public: &Keypair, key_id: i32) -> Self {
        let private_key = PrivateKey::deserialize(
            &identity_public.secret.to_bytes()
        ).expect("TODO: yeet message");

        let public_key = private_key.public_key().unwrap();

        let mut signal_public = [0u8; 33];
        signal_public[1..].copy_from_slice(public_key.public_key_bytes().unwrap());
        signal_public[0] = 5;

        let signature = private_key.calculate_signature(
            &signal_public, &mut OsRng
        ).unwrap();

        Self {
            key_pair: KeyPair::new(public_key, private_key),
            signature,
            key_id,
        }
    }
}

pub struct PublicKeyWrapper(pub PublicKey);

impl Default for Keypair {
    fn default() -> Self {
        let secret = StaticSecret::new(OsRng);
        let public = PublicKey::from(&secret);

        Self {
            public,
            secret,
        }
    }
}

impl Keypair {
    pub fn exchange<T>(&mut self, their_key: T) -> SharedSecret
    where
        T: Into<PublicKeyWrapper>,
    {
        let public_key_wrapper: PublicKeyWrapper = their_key.into();
        self.secret.diffie_hellman(&public_key_wrapper.0)
    }

}

pub fn verify_signature(their_key: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
    let public = libsignal_protocol::PublicKey::from_djb_public_key_bytes(their_key)?;
    Ok(public.verify_signature(message, signature).unwrap())
}

pub fn sign(key: &[u8], message: &[u8]) -> Result<Box<[u8]>> {
    let private = libsignal_protocol::PrivateKey::deserialize(key)?;
    Ok(private.calculate_signature(message, &mut OsRng)?)
}

impl From<[u8; 32]> for PublicKeyWrapper {
    fn from(input: [u8; 32]) -> Self {
        PublicKeyWrapper(PublicKey::try_from(input).unwrap())
    }
}

impl From<&'_ [u8]> for PublicKeyWrapper {
    fn from(input: &'_ [u8]) -> Self {
        let input: [u8; 32] = input.try_into().unwrap();
        input.try_into().unwrap()
    }
}
