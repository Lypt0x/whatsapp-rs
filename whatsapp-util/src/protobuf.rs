use serde::{Deserialize, Deserializer};

pub mod version;
pub mod whatsapp;
pub mod adv_message;

pub const MESSAGE_HEADER: [u8; 2] = [6u8, 0u8];
pub const SIGNATURE_HEADER: [u8; 2] = [6u8, 1u8];

pub(crate) fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}
