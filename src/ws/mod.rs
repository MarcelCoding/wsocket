use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::io::{split, AsyncRead, AsyncWrite, ReadHalf, WriteHalf};
use tokio::sync::broadcast;

use crate::Close;

mod read;
mod write;

pub struct WebSocket<IO> {
  io: IO,
  max_payload_len: usize,
  #[cfg(feature = "client")]
  masking: bool,
  closed: Arc<AtomicBool>,
  close: broadcast::Sender<Close>,
}

impl<IO> WebSocket<IO> {
  #[inline]
  pub fn server(io: IO, max_payload_len: usize) -> Self {
    Self {
      io,
      max_payload_len,
      #[cfg(feature = "client")]
      masking: false,
      closed: Arc::new(AtomicBool::new(false)),
      close: broadcast::Sender::new(1),
    }
  }

  #[inline]
  #[cfg(feature = "client")]
  pub fn client(io: IO, max_payload_len: usize, masking: bool) -> Self {
    Self {
      io,
      max_payload_len,
      masking,
      closed: Arc::new(AtomicBool::new(false)),
      close: broadcast::Sender::new(1),
    }
  }

  pub fn is_closed(&self) -> bool {
    self.closed.load(Ordering::SeqCst)
  }

  fn set_closed(&self, close: Close) {
    self.closed.store(true, Ordering::SeqCst);
    let _ = self.close.send(close);
  }
}

impl<IO: AsyncWrite + AsyncRead> WebSocket<IO> {
  pub fn split(self) -> (WebSocket<ReadHalf<IO>>, WebSocket<WriteHalf<IO>>) {
    let (read, write) = split(self.io);
    (
      WebSocket {
        io: read,
        max_payload_len: self.max_payload_len,
        #[cfg(feature = "client")]
        masking: self.masking,
        closed: self.closed.clone(),
        close: self.close.clone(),
      },
      WebSocket {
        io: write,
        max_payload_len: self.max_payload_len,
        #[cfg(feature = "client")]
        masking: self.masking,
        closed: self.closed,
        close: self.close,
      },
    )
  }
}
