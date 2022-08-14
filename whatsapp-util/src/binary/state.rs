#[derive(Default, PartialEq, Copy, Clone)]
pub enum State {
	#[default]
	Hello,
	Handshake,
	Connected,
	Reconnect,
	Closed,
}

impl State {
	pub fn is_default(&self) -> bool {
		*self == State::default()
	}
}