use anyhow::bail;
use crate::stream::{State, Stream};

use crate::Result;
use crate::util::error::Error;

impl Stream<'_> {
	pub async fn process_node<T>(&mut self, input: T) -> Result<()>
		where
			T: AsRef<[u8]>
	{
		// We decode nodes as soon as we arrived in Iq and the states above
		let nodes = self.client.session.decode(input)?;

		for node in nodes {
			self.digest(node).await?
		}

		Ok(())
	}

	pub async fn process_binary<T>(&mut self, input: T) -> Result<()>
		where
			T: AsRef<[u8]>
	{
		match input.as_ref() {
			// We usually receive this message when the server has decided to close this connection
			// TODO: put this in const context
			[0x88, 0x02, 0x03, 0xF3] => bail!(Error::WsClose),

			// Their message is a response of our hello identified by two zeros
			// So we're going to do a login with the decoded frame
			// TODO: find a better way to identify hello frames
			[0, 0, ..] => Ok({
				let decoded = self.client.session.decode_binary(input).swap_remove(0);
				self.login(&decoded).await?
			}),

			_ => bail!(Error::UnexpectedMessage)
		}
	}

	pub async fn process<T>(&mut self, payload: T) -> Result<()>
		where
			T: AsRef<[u8]>
	{
		match self.client.state {
			// This state should never be reached:
			// As soon as the connection is established, the websocket won't send anything until our hello is sent
			State::Hello => bail!(Error::WrongState),

			State::Handshake => self.process_binary(payload).await,
			State::Connected => self.process_node(payload).await,

			_ => Ok(())
		}
	}

}