use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::io::{split, AsyncRead, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};
use tracing::{error, info};

use crate::error::WSocketResult;
use crate::frame::{Frame, OpCode};
use crate::{CloseCode, Message, WSocketError};

pub struct WebSocket<IO> {
  io: IO,
  max_payload_len: usize,
  #[cfg(feature = "client")]
  masking: bool,
  closed: Arc<AtomicBool>,
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
    }
  }

  pub fn is_closed(&self) -> bool {
    self.closed.load(Ordering::Relaxed)
  }

  fn set_closed(&self) {
    self.closed.store(true, Ordering::Relaxed)
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
      },
      WebSocket {
        io: write,
        max_payload_len: self.max_payload_len,
        #[cfg(feature = "client")]
        masking: self.masking,
        closed: self.closed,
      },
    )
  }
}

impl<W: Unpin + AsyncWrite> WebSocket<W> {
  pub async fn send(&mut self, message: Message<'_>) -> WSocketResult<()> {
    if self.is_closed() {
      return Err(WSocketError::NotConnected)?;
    }

    let res = match message {
      Message::Binary(data) => {
        let frame = Frame::new(true, OpCode::Binary, data);
        self.send_frame(frame).await
      }
      Message::Close { code, reason } => {
        let buf = encode_close_body(code, reason);
        let frame = Frame::new(true, OpCode::Close, &buf);
        let res = self.send_frame(frame).await;
        self.set_closed();
        info!("Marking write channel as closed");
        res
      }
    };

    // set stream as closed and send close frame, if error wan't a io error
    if let Err(err) = &res {
      match err {
        WSocketError::Io(_) => {}
        _ => {
          let buf = encode_close_body(CloseCode::InternalError, None);
          let frame = Frame::new(true, OpCode::Close, &buf);
          if let Err(err) = self.send_frame(frame).await {
            error!("Failed to send close frame: {:?}", err);
          }
        }
      }
      self.set_closed();
      info!("Marking write channel as closed");
    }

    res
  }

  async fn send_frame(&mut self, frame: Frame<'_>) -> WSocketResult<()> {
    if frame.data.len() > self.max_payload_len {
      return Err(WSocketError::PayloadTooLarge);
    }

    #[cfg(not(feature = "client"))]
    frame.write_without_mask(&mut self.io).await?;

    #[cfg(feature = "client")]
    if self.masking {
      let mask = rand::random::<u32>().to_ne_bytes();
      frame.write_with_mask(&mut self.io, mask).await?;
    } else {
      frame.write_without_mask(&mut self.io).await?;
    }

    self.io.flush().await?;

    Ok(())
  }

  pub async fn flush(&mut self) -> WSocketResult<()> {
    self.io.flush().await?;
    Ok(())
  }
}

impl<R: Unpin + AsyncRead> WebSocket<R> {
  pub async fn recv<'a>(&mut self, buf: &'a mut [u8]) -> WSocketResult<Message<'a>> {
    if self.is_closed() {
      return Err(WSocketError::NotConnected)?;
    }

    let event = self.recv_message(buf).await;

    // set connection to closed
    if let Ok(Message::Close { .. }) | Err(..) = event {
      info!("marking read channel as closed");
      self.set_closed();
    }

    event
  }

  async fn recv_message<'a>(&mut self, buf: &'a mut [u8]) -> WSocketResult<Message<'a>> {
    let frame = Frame::read(&mut self.io, buf, self.max_payload_len).await?;

    if !frame.fin {
      return Err(WSocketError::FramedMessagesAreNotSupported);
    }

    match frame.opcode {
      OpCode::Continuation => Err(WSocketError::FramedMessagesAreNotSupported),
      OpCode::Text => Err(WSocketError::TextFramesAreNotSupported),
      OpCode::Binary => Ok(Message::Binary(frame.data)),
      OpCode::Close => Ok(parse_close_body(frame.data)?),
      OpCode::Ping => Err(WSocketError::PingFramesAreNotSupported),
      OpCode::Pong => Err(WSocketError::PongFramesAreNotSupported),
    }
  }
}

fn encode_close_body(code: CloseCode, reason: Option<&str>) -> Vec<u8> {
  if let Some(reason) = reason {
    let mut buf = Vec::with_capacity(2 + reason.len());
    buf.copy_from_slice(&(code as u16).to_be_bytes());
    buf.copy_from_slice(reason.as_ref());
    buf
  } else {
    let mut buf = Vec::with_capacity(2);
    buf.copy_from_slice(&(code as u16).to_be_bytes());
    buf
  }
}

fn parse_close_body(msg: &[u8]) -> WSocketResult<Message> {
  let code = msg
    .get(..2)
    .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]))
    .unwrap_or(1000);

  match code {
    1000..=1003 | 1007..=1011 | 1015 | 3000..=3999 | 4000..=4999 => {
      let msg = msg.get(2..).map(std::str::from_utf8).transpose()?;

      Ok(Message::Close {
        code: code.into(),
        reason: msg,
      })
    }
    code => Err(WSocketError::InvalidCloseCode(code)),
  }
}
