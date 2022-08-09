pub use ed25519_dalek::{Keypair as EdKeypair, Signature};
use libsignal_protocol::{KeyPair, PrivateKey};
use rand_core::OsRng;
pub use x25519_dalek::{PublicKey, SharedSecret};
use x25519_dalek::StaticSecret;

pub struct Keypair {
    pub public: PublicKey,
    pub secret: StaticSecret,

    shared_container: Vec<SharedSecret>,
}

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
            shared_container: vec![],
        }
    }
}

impl Keypair {
    pub fn exchange<'a, T>(&mut self, their_key: T) -> &SharedSecret
    where
        T: Into<PublicKeyWrapper>,
    {
        let public_key_wrapper: PublicKeyWrapper = their_key.into();
        self.shared_container
            .push(self.secret.diffie_hellman(&public_key_wrapper.0));
        self.shared_container.last().unwrap()
    }

    pub fn exchanges(&self) -> &[SharedSecret] {
        self.shared_container.as_slice()
    }

    pub fn clear_exchanges(&mut self) {
        self.shared_container.clear();
    }
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
