use tokio::io::AsyncRead;
use tokio::select;
use tracing::info;

use crate::frame::{Frame, OpCode};
use crate::{Close, CloseCode};
use crate::{Message, WSocketError, WSocketResult, WebSocket};

impl<R: Unpin + AsyncRead> WebSocket<R> {
  pub async fn recv<'a>(&mut self, buf: &'a mut [u8]) -> WSocketResult<Message<'a>> {
    if self.is_closed() {
      return Err(WSocketError::NotConnected)?;
    }

    let mut close = self.close.subscribe();

    select! {
      result = self.recv_message(buf) => {
        match result {
          Err(WSocketError::ConnectionClosed(ref close)) => {
            info!("marking read channel as closed");
            self.set_closed(close.clone());
          },
          Err(ref err) => {
            let close = Close::new(
              err.close_code().unwrap_or(CloseCode::InternalError),
              Some(format!("{}", err))
            );
            self.set_closed(close);
          },
          _ => {}
        }
        result
      },
      result = close.recv() => Err(WSocketError::ConnectionClosed(result.unwrap())),
    }
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
      OpCode::Close => Err(WSocketError::ConnectionClosed(Close::parse(
        frame.data,
      )?)),
      OpCode::Ping => Ok(Message::Ping(frame.data)),
      OpCode::Pong => Ok(Message::Pong(frame.data)),
    }
  }
}
