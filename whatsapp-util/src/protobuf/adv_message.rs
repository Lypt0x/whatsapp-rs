use bytebuffer::ByteBuffer;
use protobuf::Message;
use serde_json::Value;
use crate::model::Credentials;
use crate::protobuf::whatsapp::ADVSignedDeviceIdentity;
pub use crate::Result;

pub trait AccountMessageFormer {
	fn form_message(&self, credentials: &Credentials) -> Vec<u8>;
	fn sign(&mut self, credentials: &Credentials) -> Result<()>;
	fn without_key(&self) -> Self;
}

impl TryInto<Value> for ADVSignedDeviceIdentity {
	type Error = anyhow::Error;

	fn try_into(self) -> Result<Value> {
		Ok(serde_json::to_value(&self.write_to_bytes()?)?)
	}
}

impl AccountMessageFormer for ADVSignedDeviceIdentity {
	fn form_message(&self, credentials: &Credentials) -> Vec<u8> {
		let mut buffer = ByteBuffer::from_bytes(&crate::protobuf::MESSAGE_HEADER);
		buffer.write_bytes(&self.details.as_ref().unwrap());
		buffer.write_bytes(credentials.identity_keypair.public.as_bytes());
		buffer.to_bytes()
	}

	fn sign(&mut self, credentials: &Credentials) -> Result<()> {
		let mut buffer = ByteBuffer::from_bytes(&crate::protobuf::SIGNATURE_HEADER);
		buffer.write_bytes(self.details());
		buffer.write_bytes(credentials.identity_keypair.public.as_bytes());
		buffer.write_bytes(self.accountSignatureKey());
		let result = buffer.to_bytes();

		let signature = crate::security::keypair::sign(&credentials.identity_keypair.secret.to_bytes(), result.as_slice())?;
		self.deviceSignature = signature.into_vec().into();

		Ok(())
	}

	fn without_key(&self) -> Self {
		Self {
			details: self.details.clone().into(),
			accountSignatureKey: None,
			accountSignature: self.accountSignature.clone().into(),
			deviceSignature: self.deviceSignature.clone().into(),
			special_fields: Default::default()
		}
	}
}