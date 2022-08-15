use std::time::SystemTime;
use crate::binary::node::{DataExt, Node};

pub struct MediaConnection {
	pub auth: String,
	pub ttl: i64,
	pub max_buckets: u64,
	pub timestamp: u128,
	pub hosts: Vec<String>,
}

impl MediaConnection {
	pub fn new(auth: String, ttl: i64, max_buckets: u64, timestamp: u128, hosts: Vec<String>) -> Self {
		Self {
			auth,
			ttl,
			max_buckets,
			timestamp,
			hosts,
		}
	}
}

impl From<Node> for MediaConnection {

	fn from(value: Node) -> Self {
		let media_connection = value.find_description("media_conn").unwrap();
		let auth = media_connection["auth"].as_str().unwrap().to_string();
		let ttl = media_connection["ttl"].as_i64().unwrap();
		let max_buckets = media_connection["max_buckets"].as_u64().unwrap();
		let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();

		let hosts: Vec<String> = media_connection["hosts"].as_array().unwrap().iter()
			.map(|host| host.attribute("hostname").unwrap().as_str().unwrap().to_owned()).collect();

		Self {
			auth,
			ttl,
			max_buckets,
			timestamp,
			hosts,
		}
	}
}