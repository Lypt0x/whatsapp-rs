pub use x25519_dalek::{PublicKey, ReusableSecret, SharedSecret};

pub struct Keypair {
    pub public: PublicKey,
    secret: ReusableSecret,

    shared_container: Vec<SharedSecret>
}

impl Default for Keypair {
    fn default() -> Self {
        use rand_core::OsRng;

        let secret = ReusableSecret::new(OsRng);
        let public = PublicKey::from(&secret);

        Self { public, secret, shared_container: vec![] }
    }
}

impl Keypair {
    pub fn exchange(&mut self, their_key: &PublicKey) -> &SharedSecret {
        self.shared_container.push(self.secret.diffie_hellman(their_key));
        self.shared_container.last().unwrap()
    }

    pub fn exchanges(&self) -> &[SharedSecret] {
        self.shared_container.as_slice()
    }

    pub fn clear_exchanges(&mut self) {
        self.shared_container.clear();
    }
}