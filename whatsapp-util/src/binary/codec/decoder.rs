use super::*;

pub struct NodeDecoder {
	pub(super) buffer: ByteBuffer
}

impl NodeDecoder {
	pub fn decode(&mut self) -> Result<Node> {
		let token = self.buffer.read_u8() & 2;

		if token != 0 {
			let data = deflate::deflate_bytes({
				let remaining = self.buffer.len() - self.buffer.get_rpos();
				&self.buffer.read_bytes(remaining)
			});

			self.buffer = ByteBuffer::from_bytes(&data);
		}

		self.read_node()
	}

	fn read_node(&mut self) -> Result<Node> {
		let token = self.buffer.read_u8() as u32;
		let size = self.read_size(token);

		if size == 0 {
			bail!("No node available")
		}

		let description = self.read_string().unwrap();
		let attributes = self.read_attribute(size)?;

		Ok(if size % 2 != 0 {
			Node::from_attributes(description, attributes)
		} else {
			Node::new(description, attributes, self.read(false)?)
		})
	}

	fn read_string_from_token(&mut self, token: i32) -> &'static str {
		if token < tag::DICTIONARY_ZERO || token > tag::DICTIONARY_THREE {
			return token::SINGLE_BYTE[(token - 1) as usize];
		}

		let delta = (token::DOUBLE_BYTE.len() / 4) as i32 * (token - tag::DICTIONARY_ZERO);
		token::DOUBLE_BYTE[(self.buffer.read_u8() as i32 + delta) as usize]
	}

	fn read_attribute(&mut self, size: u32) -> Result<HashMap<String, Value>> {
		let mut map = HashMap::new();

		(2..size).step_by(2).try_for_each(|_| {
			let key = self.read_string()?;
			let value = self.read(true)?;
			map.insert(key, value);

			Result::<_>::Ok(())
		})?;

		Ok(map)
	}

	fn read_string(&mut self) -> Result<String> {
		if let Value::String(token) = self.read(true)? {
			return Ok(token);
		}

		bail!("Could not read string")
	}

	fn read(&mut self, parse: bool) -> Result<Value> {
		let tag = self.buffer.read_u8() as i32;

		Ok(match tag {
			tag::LIST_EMPTY => Value::Null,
			tag::COMPANION_JID => self.read_companion_jid()?,
			tag::LIST_EIGHT => {
				let size = self.buffer.read_u8() as u32;
				self.read_list(size)?
			}
			tag::LIST_SIXTEEN => {
				let size = self.buffer.read_u16() as u32;
				self.read_list(size)?
			}
			tag::JID_PAIR => self.read_jid_pair()?,
			tag::HEX_EIGHT => self.read_hex_string(),
			tag::BINARY_EIGHT => {
				let size = self.buffer.read_u8() as u32;
				self.read_string_until(size, parse)?
			}
			tag::BINARY_TWENTY => {
				let size = self.read_string_custom_length();
				self.read_string_until(size, parse)?
			}
			tag::BINARY_THIRTY_TWO => {
				let size = self.buffer.read_u16() as u32;
				self.read_string_until(size, parse)?
			}
			tag::NIBBLE_EIGHT => self.read_nibble(),
			_ => {
				let token = self.read_string_from_token(tag);
				Value::String(token.to_owned())
			}
		})
	}

	fn read_nibble(&mut self) -> Value {
		let number = self.buffer.read_u8() as u32;

		let start = number >> 7;
		let end = 127 & number;

		let output = String::with_capacity((2 * end - start) as usize);

		let output = self.read_string_mode(output, false);

		Value::String(output)
	}

	fn read_string_custom_length(&mut self) -> u32 {
		((15 & self.buffer.read_u8() as u32) << 16)
			+ ((self.buffer.read_u8() as u32) << 8)
			+ self.buffer.read_u8() as u32
	}

	fn read_string_until(&mut self, size: u32, parse: bool) -> Result<Value> {
		let data = self.buffer.read_bytes(size as usize);
		Ok(if parse {
			Value::String(String::from_utf8(data)?)
		} else {
			// SAFETY: We do not handle non-parseable strings at all
			// So, whether the String is corrupted or not, we don't need to worry
			unsafe { Value::String(String::from_utf8_unchecked(data)) }
		})
	}

	fn read_hex_string(&mut self) -> Value {
		let number = self.buffer.read_u8() as u32;

		let start = number >> 7;
		let end = 127 & number;

		let output = String::with_capacity((2 * end - start) as usize);

		let output = self.read_string_mode(output, true);

		Value::String(output)
	}

	fn read_string_mode(&mut self, mut output: String, hex: bool) -> String {
		let mut index = 0_usize;

		loop {
			if index >= output.capacity() - 1 {
				break;
			}

			let token = self.buffer.read_u8() as u32;
			if hex {
				output.insert(index, token::HEX[(token >> 4) as usize]);
				output.insert(index + 1, token::HEX[(15 & token) as usize]);
			} else {
				output.insert(index, token::NUMBERS[(token >> 4) as usize]);
				output.insert(index + 1, token::NUMBERS[(15 & token) as usize]);
			}

			index += 2
		}

		output
	}

	fn read_list(&mut self, size: u32) -> Result<Value> {
		let mut list = vec![];
		for _ in 0..size {
			list.push(serde_json::to_value(self.read_node()?)?)
		}

		Ok(Value::Array(list))
	}

	fn read_jid_pair(&mut self) -> Result<Value> {
		Ok(match self.read(true)? {
			Value::String(encoded) => serde_json::to_value({
				let server = Server::of(&self.read_string()?).unwrap_or(Server::Whatsapp);
				ContactJid::from_complex(encoded, server)?
			})?,
			Value::Null => serde_json::to_value({
				let server = Server::of(&self.read_string()?).unwrap_or(Server::Whatsapp);
				ContactJid::from_complex(String::new(), server)?
			})?,
			_ => bail!("Could not read jid pair"),
		})
	}

	fn read_companion_jid(&mut self) -> Result<Value> {
		let agent = self.buffer.read_u8() as u32;
		let device = self.buffer.read_u8() as u32;
		let user = self.read_string()?;

		Ok(serde_json::to_value(ContactJid::from_companion(
			user, device, agent,
		))?)
	}

	fn read_size(&mut self, token: u32) -> u32 {
		match token as i32 {
			tag::LIST_EIGHT => self.buffer.read_u8() as u32,
			_ => self.buffer.read_u16() as u32,
		}
	}
}