pub mod codec;
pub mod node;
pub mod handshake;
pub mod session;

pub const PROTOCOL: [u8; 32] = *b"Noise_XX_25519_AESGCM_SHA256\0\0\0\0";
pub const PROLOGUE: [u8; 4] = [87, 65, 5, 2];
