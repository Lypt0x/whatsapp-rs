pub use crate::result::Error;
pub use anyhow::Result;

pub fn iv_as_nonce(iv: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    let src: [u8; 8] = iv.to_be_bytes();

    nonce[4..].copy_from_slice(&src);
    nonce
}

pub fn encrypt<I>(key: [u8; 32], hash: [u8; 32], nonce: [u8; 12], input: I) -> Result<Vec<u8>>
where
    I: AsRef<[u8]>,
{
    use aes_gcm::{
        aead::{Aead, NewAead, Payload},
        Aes256Gcm, Key, Nonce,
    };

    let key = Key::from_slice(key.as_ref());
    let nonce = Nonce::from_slice(nonce.as_ref());
    let cipher = Aes256Gcm::new(key);
    let payload = Payload {
        msg: input.as_ref(),
        aad: hash.as_ref(),
    };

    Ok(cipher
        .encrypt(nonce, payload)
        .map_err(Error::AesCipherFail)?)
}

pub fn decrypt<I>(key: [u8; 32], hash: [u8; 32], nonce: [u8; 12], input: I) -> Result<Vec<u8>>
where
    I: AsRef<[u8]>,
{
    use aes_gcm::{
        aead::{Aead, NewAead, Payload},
        Aes256Gcm, Key, Nonce,
    };

    let key = Key::from_slice(&key);
    let nonce = Nonce::from_slice(nonce.as_ref());
    let cipher = Aes256Gcm::new(key);
    let payload = Payload {
        msg: input.as_ref(),
        aad: &hash,
    };

    Ok(cipher
        .decrypt(nonce, payload)
        .map_err(Error::AesCipherFail)?)
}

pub fn decrypt_no_hash<I>(key: [u8; 32], nonce: [u8; 12], input: I) -> Result<Vec<u8>>
where
    I: AsRef<[u8]>,
{
    use aes_gcm::{
        aead::{Aead, NewAead},
        Aes256Gcm, Key, Nonce,
    };

    let key = Key::from_slice(&key);
    let nonce = Nonce::from_slice(nonce.as_ref());
    let cipher = Aes256Gcm::new(key);

    Ok(cipher
        .decrypt(nonce, input.as_ref())
        .map_err(Error::AesCipherFail)?)
}
