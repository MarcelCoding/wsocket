pub use close::{Close, CloseCode};
pub use error::WSocketError;
pub use error::WSocketResult;
pub use ws::WebSocket;

mod error;
mod frame;
mod ws;

mod close;

pub enum Message<'a> {
  Binary(&'a [u8]),
  Text(&'a str),
  Ping(&'a [u8]),
  Pong(&'a [u8]),
}
