pub use close::{Close, CloseCode};
pub use error::WSocketError;
pub use error::WSocketResult;
#[cfg(all(feature = "handshake", feature = "client"))]
pub use handshake::handshake;
#[cfg(feature = "upgrade")]
pub use upgrade::{is_upgrade_request, upgrade};
pub use ws::WebSocket;

mod close;
mod error;
mod frame;
mod ws;

#[cfg(all(feature = "handshake", feature = "client"))]
mod handshake;

#[cfg(feature = "upgrade")]
mod upgrade;

pub enum Message<'a> {
  Binary(&'a [u8]),
  Text(&'a str),
  Ping(&'a [u8]),
  Pong(&'a [u8]),
}
