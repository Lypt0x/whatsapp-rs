use crate::security::keypair::{Keypair, SignedKeypair};
use rand::Rng;

pub struct Credentials {
    pub noise_keypair: Keypair,
    pub ephemeral_keypair: Keypair,
    pub identity_keypair: Keypair,
    pub signed_keypair: SignedKeypair,
    pub companion_secret: [u8; 32],

    pub(crate) registration_id: u32,
}

impl Credentials {
    
    pub fn noise_public(&self) -> [u8; 32] {
        self.noise_keypair.public.to_bytes()
    }
    
    pub fn noise_private(&self) -> [u8; 32] {
        self.noise_keypair.secret.to_bytes()
    }
    
    pub fn identity_public(&self) -> [u8; 32] {
        self.identity_keypair.public.to_bytes()
    }
    
    pub fn identity_private(&self) -> [u8; 32] {
        self.identity_keypair.secret.to_bytes()
    }

    pub fn ephemeral_public(&self) -> [u8; 32] {
        self.ephemeral_keypair.public.to_bytes()
    }

    pub fn ephemeral_private(&self) -> [u8; 32] {
        self.ephemeral_keypair.secret.to_bytes()
    }
    
}

impl Default for Credentials {
    fn default() -> Self {
        let identity_keypair = Keypair::default();

        Self {
            noise_keypair: Default::default(),
            ephemeral_keypair: Default::default(),
            signed_keypair: SignedKeypair::new(&identity_keypair, 1),
            registration_id: rand::thread_rng().gen_range(0..16380) + 1,
            companion_secret: Keypair::default().public.to_bytes(),
            identity_keypair,
        }
    }
}
