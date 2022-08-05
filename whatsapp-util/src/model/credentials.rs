use crate::binary::session::Session;
use crate::security::keypair::{Keypair, SignedKeypair};
use rand::Rng;

pub struct Credentials {
    pub noise_keypair: Keypair,
    pub ephemeral_keypair: Keypair,
    pub identity_keypair: Keypair,
    pub signed_keypair: SignedKeypair,

    pub session: Option<Session>,

    pub(crate) registration_id: u32,
}

impl Default for Credentials {
    fn default() -> Self {
        let identity_keypair = Keypair::default();

        Self {
            noise_keypair: Default::default(),
            ephemeral_keypair: Default::default(),
            signed_keypair: SignedKeypair::new(identity_keypair.public.to_bytes(), 1),
            registration_id: rand::thread_rng().gen_range(0..16380) + 1,
            identity_keypair,
            session: None,
        }
    }
}
