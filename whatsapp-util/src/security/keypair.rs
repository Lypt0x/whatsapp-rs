use ed25519_dalek::Signer;
pub use ed25519_dalek::{Keypair as EdKeypair, Signature};
use rand_core::OsRng;
pub use x25519_dalek::{PublicKey, ReusableSecret, SharedSecret};

pub struct Keypair {
    pub public: PublicKey,
    secret: ReusableSecret,

    shared_container: Vec<SharedSecret>,
}

pub struct SignedKeypair {
    pub key_pair: EdKeypair,
    pub signature: Signature,
    pub key_id: i32,
}

impl SignedKeypair {
    pub fn new(identity_public: [u8; 32], key_id: i32) -> Self {
        let sign_keys = EdKeypair::generate(&mut OsRng);
        let mut pub_key = [0u8; 33];

        pub_key[1..].copy_from_slice(&identity_public);
        pub_key[0] = 5;

        let signature = sign_keys.sign(&pub_key);

        Self {
            key_pair: sign_keys,
            signature,
            key_id,
        }
    }
}

pub struct PublicKeyWrapper(pub PublicKey);

impl Default for Keypair {
    fn default() -> Self {
        let secret = ReusableSecret::new(OsRng);
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
        input.try_into().unwrap()
    }
}
