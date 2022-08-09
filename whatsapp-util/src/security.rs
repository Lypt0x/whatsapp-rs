pub mod aes;
pub mod hash;
pub mod hkdf;
pub mod keypair;

pub use base64;

pub use crate::Result;
pub use crate::util::error::Error;

pub trait AsNonce {
	fn nonce(self) -> [u8; 12];
	fn get_increment_nonce_mut(&mut self) -> [u8; 12];
}

impl AsNonce for u64 {
	fn nonce(self) -> [u8; 12] {
		let mut nonce = [0u8; 12];
		let src: [u8; 8] = self.to_be_bytes();

		nonce[4..].copy_from_slice(&src);
		nonce
	}

	fn get_increment_nonce_mut(&mut self) -> [u8; 12] {
		let previous = *self;
		*self += 1;

		previous.nonce()
	}
}