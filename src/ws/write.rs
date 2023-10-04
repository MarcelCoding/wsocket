use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::select;
use tracing::{error, info};

use crate::frame::{Frame, OpCode};
use crate::{Close, CloseCode, Message, WSocketError, WSocketResult, WebSocket};

impl<W: Unpin + AsyncWrite> WebSocket<W> {
  pub async fn send(&mut self, message: Message<'_>) -> WSocketResult<()> {
    if self.is_closed() {
      return Err(WSocketError::NotConnected)?;
    }

    let frame = match message {
      Message::Binary(data) => Frame::new(true, OpCode::Binary, data),
      Message::Text(text) => Frame::new(true, OpCode::Text, text.as_bytes()),
      Message::Ping(data) => Frame::new(true, OpCode::Ping, data),
      Message::Pong(data) => Frame::new(true, OpCode::Pong, data),
    };

    let mut close = self.close.subscribe();

    // aboard send if connection got closed
    let result = select! {
      result = self.send_frame(frame) => {
        if let Err(ref err) = result {
          let close = Close::new(
            err.close_code().unwrap_or(CloseCode::InternalError),
            Some(format!("{}", err))
          );

          if !err.is_io_error() {
            if let Err(err) = self.close(close).await {
              error!("Failed to send close frame: {}", err);
            }
          } else {
            info!("Marking write channel as closed");
            self.set_closed(close);
          }
        }
        result
      },
      // TODO: is unwrap ok here?
      result = close.recv() => return Err(WSocketError::ConnectionClosed(result.unwrap())),
    };

    // mark stream as closed and send close frame, if error wasn't an io error

    result
  }

  pub async fn close(&mut self, close: Close) -> WSocketResult<()> {
    let buf = close.encode()?;
    let frame = Frame::new(true, OpCode::Close, &buf);
    self.set_closed(close.clone());
    self.send_frame(frame).await
  }

  async fn send_frame(&mut self, frame: Frame<'_>) -> WSocketResult<()> {
    if frame.data.len() > self.max_payload_len {
      return Err(WSocketError::PayloadTooLarge);
    }

    #[cfg(not(feature = "client"))]
    frame.write_without_mask(&mut self.io).await?;

    #[cfg(feature = "client")]
    if self.masking {
      let mask = rand::random();
      frame.write_with_mask(&mut self.io, mask).await?;
    } else {
      frame.write_without_mask(&mut self.io).await?;
    }

    self.io.flush().await?;

    Ok(())
  }
}
