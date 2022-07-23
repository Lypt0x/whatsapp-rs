use crate::auth::Keypair;
use crate::auth::session::Session;

pub struct Credentials {
    pub noise_keypair: Keypair,
    pub ephemeral_keypair: Keypair,
    pub identity_keypair: Keypair,

    pub session: Option<Session>
}