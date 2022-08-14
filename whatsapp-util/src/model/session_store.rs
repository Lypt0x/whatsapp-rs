use crate::model::ContactJid;
use crate::protobuf::whatsapp::ADVSignedDeviceIdentity;
use crate::security::{aes, AsNonce, hkdf};
use crate::Result;

// TODO: Support asynchronous r/w
#[derive(Debug, Default, Clone)]
pub struct SessionStore {
	pub decode_key: [u8; 32],
	pub encode_key: [u8; 32],

	pub read_cnt: u64,
	pub write_cnt: u64,

	pub companion: Option<ContactJid>,
	pub companion_identity: Option<ADVSignedDeviceIdentity>,
}

pub enum TrafficType {
	Ingoing,
	Outgoing
}

impl SessionStore {
	pub fn count_nonce(&mut self, traffic: TrafficType) -> [u8; 12] {
		match traffic {
			TrafficType::Ingoing => {
				self.read_cnt += 1;
				(self.read_cnt-1).nonce()
			},

			TrafficType::Outgoing => {
				self.write_cnt += 1;
				(self.write_cnt-1).nonce()
			}
		}
	}
	
	pub fn encrypt(&mut self, input: &[u8]) -> Result<Vec<u8>> {
		aes::encrypt_no_hash(
			self.encode_key,
			self.count_nonce(TrafficType::Ingoing),
			input,
		)
	}
	
	pub fn decrypt(&mut self, input: &[u8]) -> Result<Vec<u8>> {
		aes::decrypt_no_hash(
			self.decode_key,
			self.count_nonce(TrafficType::Outgoing),
			input,
		)
	}

	pub fn update(&mut self, salt: [u8; 32]) -> Result<()> {
		let expanded = hkdf::expand_extract(salt, &[]);
		self.encode_key = expanded.as_ref()[..32].try_into()?;
		self.decode_key = expanded.as_ref()[32..].try_into()?;
		Ok(())
	}
}