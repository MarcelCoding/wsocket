#[cfg(any(feature = "handshake", feature = "upgrade"))]
use hyper::upgrade::Upgraded;
#[cfg(any(feature = "handshake", feature = "upgrade"))]
use hyper_util::rt::TokioIo;

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

#[cfg(feature = "upgrade")]
mod idk;

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

#[cfg(any(feature = "handshake", feature = "upgrade"))]
type UpgradedWebsocketIo = TokioIo<Upgraded>;
