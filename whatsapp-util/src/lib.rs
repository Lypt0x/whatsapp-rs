pub mod binary;
pub mod model;
pub mod protobuf;
pub mod security;
pub mod util;

pub use anyhow::Result;

#[cfg(test)]
mod tests {
	#[test]
	pub fn encode_decode_node() {
		// TODO: encode decode test
	}

}