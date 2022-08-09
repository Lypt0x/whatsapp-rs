#[derive(Default, PartialEq)]
pub enum State {
	#[default]
	Hello,
	Handshake,
	Iq,
}

impl State {
	pub fn is_default(&self) -> bool {
		*self == State::default()
	}
}