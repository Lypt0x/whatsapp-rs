use std::collections::HashMap;
use anyhow::bail;
use whatsapp_rs_util::binary::node::{DataExt, Node, Value};
use whatsapp_rs_util::model::{contact_jid, Session, SessionStore};
use whatsapp_rs_util::protobuf::adv_message::AccountMessageFormer;
use whatsapp_rs_util::protobuf::whatsapp::{ADVDeviceIdentity, ADVSignedDeviceIdentity, ADVSignedDeviceIdentityHMAC, MessageParser};
use whatsapp_rs_util::security;
use whatsapp_rs_util::security::Error;
use crate::stream::digest::{Digest, DigestData};
use crate::Result;

pub struct Iq;

impl Iq {
	pub fn print(container: Node, session: &Session) {
		let reference = container.find_description("ref")
			.expect("Missing ref").content::<&str>().unwrap();

		let formatted_code = format!(
			"{},{noise_public},{identity_public},{companion}",
			reference,
			noise_public = security::base64::encode(session.credentials.noise_keypair.public.as_bytes()),
			identity_public = security::base64::encode(session.credentials.identity_keypair.public.as_bytes()),
			companion = security::base64::encode(session.credentials.signed_keypair.key_pair.public_key.public_key_bytes().unwrap())
		);

		qr2term::print_qr(formatted_code).unwrap();
	}

	pub fn send_confirm(node: Node, content: Value) -> Node {
		let request = Node::new(
			"iq".to_owned(),
			HashMap::from([
				("id".to_owned(), Value::String(node.attributes_clone()["id"].as_str().unwrap().to_string())),
				("type".to_owned(), Value::String("result".to_owned())),
				("to".to_owned(), Value::String(contact_jid::Server::Whatsapp.address().to_owned()))
			]),
			content
		);

		request
	}

	pub fn identify(session: &mut Session, node: Node, container: Node) -> Result<Node> {
		Self::save_companion(&container, &mut session.store)?;

		let device_identity = container.find_description("device-identity")
			.expect("Missing device-identity").content_array_nums().unwrap();

		let adv_identity = ADVSignedDeviceIdentityHMAC::parse_from_bytes(device_identity.as_slice())?;
		let adv_sign = security::hash::mac_sha256(
			session.credentials.signed_keypair.key_pair.public_key.public_key_bytes().unwrap(),
			adv_identity.details.as_ref().unwrap()
		);

		if adv_sign.ne(adv_identity.hmac.unwrap().as_slice()) {
			bail!(Error::IqInvalidSignature)
		}

		let mut account = ADVSignedDeviceIdentity::parse_from_bytes(&adv_identity.details.unwrap())?;
		let message = account.form_message(&session.credentials);

		if !security::keypair::verify_signature(account.accountSignatureKey(), &message, account.accountSignature())? {
			bail!(Error::IqInvalidMessageSignature)
		}

		account.sign(&session.credentials)?;

		let key_index = ADVDeviceIdentity::parse_from_bytes(account.details.as_ref().unwrap())?.keyIndex();

		let attributes: HashMap<String, Value> = HashMap::from([("key-index".to_owned(), key_index.into())]);
		let account_without_key_node: Value = account.without_key().try_into()?;
		let identity_node = Node::new("device-identity".to_owned(), attributes, account_without_key_node);

		let pair_device = Node::new(
			"pair-device-sign".to_owned(),
			HashMap::new(),
			Node::serialize(identity_node).unwrap()
		);

		session.store.companion_identity = account.into();

		let serialized = Node::serialize(pair_device).unwrap();
		Ok(Self::send_confirm(node, serialized))
	}

	pub fn save_companion(container: &Node, store: &mut SessionStore) -> Result<()> {
		let device_node = Node::deserialize(
			container.find_description("device").ok_or(Error::IqMissingDevice)?.clone()
		).unwrap();

		let attributes = device_node.attributes_clone();
		let companion_node = attributes.get("jid").unwrap();
		let jid = Node::parse_jid(companion_node);
		store.companion = jid.into();
		Ok(())
	}
}

impl Digest for Iq {
	fn digest(data: DigestData) -> Result<Option<DigestData>>
	{
		let DigestData { mut session, node} = data;

		// TODO: error handling
		let container = node.children().first()
			.and_then(|child| child.as_array())
			.and_then(|nodes| nodes.first()).unwrap();

		let container: Node = container.try_into()?;

		Ok(match container.description() {
			"pair-device" => {
				// Print qr code and send confirmation
				Iq::print(container, &session);
				DigestData {
					session,
					node: Iq::send_confirm(node, Value::Null)
				}.into()
			},

			"pair-success" => {
				// When nothing went wrong, we receive a pair-success node
				// Here we identify the companion, save it to the store and sending a confirmation once again
				DigestData {
					node: Iq::identify(&mut session, node, container)?,
					session,
				}.into()
			}

			_ => unreachable!()
		})
	}
}