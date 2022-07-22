pub use crate::result::Error;
pub use anyhow::Result;

pub fn encrypt<K, I>(key: K, hash: K, nonce: I, input: I) -> Result<Vec<u8>>
where
    K: AsRef<[u8; 32]>,
    I: AsRef<[u8]>
{
    use aes_gcm::{
        Aes256Gcm, Key, Nonce,
        aead::{
            Payload, Aead, NewAead
        }
    };

    let key = Key::from_slice(key.as_ref());
    let nonce = Nonce::from_slice(nonce.as_ref());
    let cipher = Aes256Gcm::new(key);
    let payload = Payload { msg: input.as_ref(), aad: hash.as_ref() };

    Ok(cipher.encrypt(nonce, payload).map_err(Error::AesCipherFail)?)
}

pub fn decrypt<K, I>(key: K, hash: K, nonce: I, input: I) -> Result<Vec<u8>>
    where
        K: AsRef<[u8; 32]>,
        I: AsRef<[u8]>
{
    use aes_gcm::{
        Aes256Gcm, Key, Nonce,
        aead::{
            Payload, Aead, NewAead
        }
    };

    let key = Key::from_slice(key.as_ref());
    let nonce = Nonce::from_slice(nonce.as_ref());
    let cipher = Aes256Gcm::new(key);
    let payload = Payload { msg: input.as_ref(), aad: hash.as_ref() };

    Ok(cipher.decrypt(nonce, payload).map_err(Error::AesCipherFail)?)
}
