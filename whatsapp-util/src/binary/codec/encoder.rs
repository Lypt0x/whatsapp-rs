use regex::Regex;
use serde_json::Map;
use super::*;

pub const I32_20_MAX_VALUE: i64 = 1048576;

pub struct NodeEncoder {
	node: Node,
	buffer: ByteBuffer
}

impl NodeEncoder {
	pub fn encode(node: Node) -> Result<Vec<u8>> {
		Self {
			node,
			buffer: ByteBuffer::new()
		}.encode_internal()
	}
}

impl NodeEncoder {
	pub fn new(node: Node) -> Self {
		Self {
			buffer: ByteBuffer::new(),
			node
		}
	}

	fn encode_internal(&mut self) -> Result<Vec<u8>> {
		self.write_node()?;
		let mut result = vec![0u8; 1 + self.buffer.len()];
		result[1..].copy_from_slice(&self.buffer.to_bytes());
		Ok(result)
	}

	fn write_node(&mut self) -> Result<()> {
		if self.node.description() == "0" {
			self.buffer.write_u8(tag::LIST_EIGHT as u8);
			self.buffer.write_u8(tag::LIST_EMPTY as u8);
			return Ok(())
		}

		self.write_i32(self.node.size())?;

		let description = self.node.description().to_owned();
		self.write_string(&description)?;

		self.write_attributes(self.node.attributes_clone().into_iter().collect())?;

		if self.node.has_content() {
			let mut inner = self.node.content_as_value().clone();
			if let Value::Object(_) = inner {
				inner = Value::Array(vec![inner]);
			}

			self.write(inner)?;
		}

		Ok(())
	}

	fn write_i32(&mut self, size: usize) -> Result<()> {
		Ok(match size {
			size if size < u8::MAX as usize + 1 => {
				self.buffer.write_u8(tag::LIST_EIGHT as u8);
				self.buffer.write_u8(size as u8);
			},

			size if size < u16::MAX as usize + 1 => {
				self.buffer.write_u8(tag::LIST_SIXTEEN as u8);
				self.buffer.write_u8(size as u8);
			},

			_ => bail!(
				"Node is unexpectedly too large ({} >= {} >= {})",
				size, u16::MAX as i32 + 1, u8::MAX as i32 + 1
			)
		})
	}

	fn write_string(&mut self, input: &str) -> Result<()> {
		if input.is_empty() {
			self.buffer.write_u8(tag::BINARY_EIGHT as u8);
			self.buffer.write_u8(tag::LIST_EMPTY as u8);
			return Ok(())
		}


		if let Some(index) = token::SINGLE_BYTE.iter().position(|&node| node == input) {
			self.buffer.write_u8(index as u8 + 1);
			return Ok(());
		}

		if self.write_double_byte_string(input).unwrap() {
			return Ok(())
		}

		let length = input.len();
		if length < 128 {
			let numbers_regex = Regex::new("[^0-9.-]+?")?;
			if !numbers_regex.is_match(input) {
				return self.write_string_token(input, tag::NIBBLE_EIGHT)
			}

			let hex_regex = Regex::new("[^0-9A-F]+?")?;
			if !hex_regex.is_match(input) {
				return self.write_string_token(input, tag::HEX_EIGHT)
			}
		}

		self.write_i64(length as i64);
		self.buffer.write_bytes(input.as_bytes());

		Ok(())
	}

	fn write_i64(&mut self, length: i64) {
		match length {
			0..=255 => {
				self.buffer.write_u8(tag::BINARY_EIGHT as u8);
				self.buffer.write_u8(length as u8);
				return
			},

			0..=I32_20_MAX_VALUE => {
				self.buffer.write_u8(tag::BINARY_TWENTY as u8);
				self.buffer.write_u8((((length as u32) >> 16) & 0xFF) as u8);
				self.buffer.write_u8((((length as u32) >> 8) & 0xFF) as u8);
				self.buffer.write_u8((length as u32 & 0xFF) as u8);
				return
			},
			_ => ()
		};

		self.buffer.write_u8(tag::BINARY_THIRTY_TWO as u8);
		self.buffer.write_u8(length as u8);
	}

	#[allow(unused)]
	fn write_jid(&mut self, jid: ContactJid) -> Result<()> {
		if jid.is_companion() {
			self.buffer.write_u8(tag::COMPANION_JID as u8);
			self.buffer.write_u8(jid.agent as u8);
			self.buffer.write_u8(jid.device as u8);
			return self.write_string(&jid.user)
		}

		self.buffer.write_u8(tag::JID_PAIR as u8);
		if !jid.user.is_empty() {
			self.write_string(&jid.user)?;
			return self.write_string(jid.server.address())
		}

		self.buffer.write_u8(tag::LIST_EMPTY as u8);
		self.write_string(jid.server.address())
	}

	fn write(&mut self, value: Value) -> Result<()> {
		Ok(match value {
			Value::Null => self.buffer.write_u8(tag::LIST_EMPTY as u8),
			Value::Bool(state) => self.write_string(if state { "true" } else { "false"})?,
			Value::Number(num) => {
				self.write_string(&num.to_string())?
			},
			Value::String(input) => {
				self.write_string(&input)?
			},
			Value::Array(input) => {
				if input.iter().all(|value| value.is_number()) {
					let bytes = serde_json::from_value::<Vec<u8>>(Value::Array(input))?;
					return Ok(self.write_bytes(&bytes))
				}
				self.write_list(serde_json::from_value(Value::Array(input))?)?
			},
			Value::Object(value) => {
				self.write_attributes(value)?
			},
		})
	}

	fn write_attributes(&mut self, attributes: Map<String, Value>) -> Result<()> {
		attributes.into_iter()
			.try_for_each(|(key, value)| {
				self.write_string(&key)?;
				self.write(value)
			})
	}

	fn write_list(&mut self, nodes: Vec<Node>) -> Result<()> {
		self.write_i32(nodes.len())?;
		nodes.into_iter().try_for_each(|node| {
			let mut container = Self::new(node);
			container.write_node()?;
			Result::<_>::Ok(self.buffer.write_bytes(&container.buffer.to_bytes()))
		})
	}

	fn write_bytes(&mut self, bytes: &[u8]) {
		self.write_i64(bytes.len() as i64);
		self.buffer.write_bytes(bytes);
	}

	fn write_double_byte_string(&mut self, input: &str) -> Result<bool> {
		if !token::DOUBLE_BYTE.contains(&input) {
			return Ok(false)
		}

		let index = token::DOUBLE_BYTE.iter().position(|&token| token == input)
			.expect("Invalid token");
		self.buffer.write_u8(Self::double_byte_string_tag(index)? as u8);
		self.buffer.write_u8((index % (token::DOUBLE_BYTE.len() / 4)) as u8);

		Ok(true)
	}

	fn double_byte_string_tag(index: usize) -> Result<i32> {
		Ok(match index / (token::DOUBLE_BYTE.len() / 4) {
			0 => tag::DICTIONARY_ZERO,
			1 => tag::DICTIONARY_ONE,
			2 => tag::DICTIONARY_TWO,
			3 => tag::DICTIONARY_THREE,
			_ => bail!("Cannot find index for quadrant {}", index)
		})
	}

	fn write_string_token(&mut self, input: &str, token: i32) -> Result<()> {
		self.buffer.write_u8(token as u8);
		self.write_string_length(input);

		let mut last_code_point = 0;
		input.chars().enumerate().try_for_each(|(idx, ch)| {
			let code_point = ch as u32;
			let bin_code_point = Self::string_code_point(token, code_point)?;

			if idx % 2 != 0 {
				last_code_point |= bin_code_point;

				self.buffer.write_u8(last_code_point as u8);
			} else {
				last_code_point = bin_code_point << 4;

				if idx == input.len() - 1 {
					last_code_point |= 15;

					self.buffer.write_u8(last_code_point as u8)
				}
			}

			Result::<_>::Ok(())
		})
	}

	fn string_code_point(token: i32, code_point: u32) -> Result<u32> {
		Ok(match (token, code_point) {
			(tag::NIBBLE_EIGHT, 45 | 46) => 10 + code_point - 45,
			(tag::HEX_EIGHT, 65..=70) => code_point - 55,
			(_, 48..=57) => code_point - 48,
			_ => bail!(
				"Invalid codepoint {} with token {}",
				code_point, token
			)
		})
	}

	fn write_string_length(&mut self, input: &str) {
		let rounded = ((input.len() as f64) / 2f64).ceil() as u32;
		if input.len() % 2 == 1 {
			self.buffer.write_u8((rounded | 0x80) as u8);
			return
		}

		self.buffer.write_u8(rounded as u8)
	}
}