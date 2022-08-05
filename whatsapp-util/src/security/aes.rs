use super::*;

use aes_gcm::{
    aead::{Aead, NewAead},
    Aes256Gcm, Key, Nonce,
};

pub fn encrypt<I>(key: [u8; 32], hash: [u8; 32], nonce: [u8; 12], input: I) -> Result<Vec<u8>>
where
    I: AsRef<[u8]>,
{
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
    let key = Key::from_slice(&key);
    let nonce = Nonce::from_slice(nonce.as_ref());
    let cipher = Aes256Gcm::new(key);

    Ok(cipher
        .decrypt(nonce, input.as_ref())
        .map_err(Error::AesCipherFail)?)
}

pub fn encrypt_no_hash<I>(key: [u8; 32], nonce: [u8; 12], input: I) -> Result<Vec<u8>>
    where
        I: AsRef<[u8]>,
{
    let key = Key::from_slice(&key);
    let nonce = Nonce::from_slice(nonce.as_ref());
    let cipher = Aes256Gcm::new(key);

    Ok(cipher
        .encrypt(nonce, input.as_ref())
        .map_err(Error::AesCipherFail)?)
}