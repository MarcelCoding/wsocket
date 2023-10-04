use std::io;
use std::str::Utf8Error;

use crate::CloseCode;
use thiserror::Error;

pub type WSocketResult<T> = Result<T, WSocketError>;

#[derive(Error, Debug)]
pub enum WSocketError {
  #[error("unknown opcode `{0}`")]
  UnknownOpCode(u8),
  #[error("reserve bit must be `0`")]
  ReserveBitMustBeNull,
  #[error("control frame must not be fragmented")]
  ControlFrameMustNotBeFragmented,
  #[error("control frame must have a payload length of 125 bytes or less")]
  ControlFrameMustHaveAPayloadLengthOf125BytesOrLess,
  #[error("payload too large")]
  PayloadTooLarge,
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error("not connected")]
  NotConnected,
  #[error("connection closed: `{0}` `{}`", .1.as_ref().unwrap_or(&"".to_string()))]
  ConnectionClosed(CloseCode, Option<String>),
  #[error("framed messages are not supported")]
  FramedMessagesAreNotSupported,
  #[error("text frames are not supported")]
  TextFramesAreNotSupported,
  #[error("ping frames are not supported")]
  PingFramesAreNotSupported,
  #[error("pong frames are not supported")]
  PongFramesAreNotSupported,
  #[error(transparent)]
  InvalidUtf8(#[from] Utf8Error),
  #[error("invalid close close `{0}`")]
  InvalidCloseCode(u16),
}

impl WSocketError {
  pub(crate) fn is_io_error(&self) -> bool {
    matches!(self, WSocketError::Io(_))
  }

  pub(crate) fn close_code(&self) -> Option<CloseCode> {
    match self {
      WSocketError::UnknownOpCode(_) => Some(CloseCode::ProtocolError),
      WSocketError::ReserveBitMustBeNull => Some(CloseCode::Unsupported),
      WSocketError::ControlFrameMustNotBeFragmented => Some(CloseCode::Unsupported),
      WSocketError::ControlFrameMustHaveAPayloadLengthOf125BytesOrLess => {
        Some(CloseCode::ProtocolError)
      }
      WSocketError::PayloadTooLarge => Some(CloseCode::MessageTooBig),
      WSocketError::Io(_) => None,
      WSocketError::NotConnected => None,
      WSocketError::ConnectionClosed(..) => None,
      WSocketError::FramedMessagesAreNotSupported => Some(CloseCode::Unsupported),
      WSocketError::TextFramesAreNotSupported => Some(CloseCode::Unsupported),
      WSocketError::PingFramesAreNotSupported => Some(CloseCode::Unsupported),
      WSocketError::PongFramesAreNotSupported => Some(CloseCode::Unsupported),
      WSocketError::InvalidUtf8(_) => Some(CloseCode::InvalidPayload),
      WSocketError::InvalidCloseCode(_) => None,
    }
  }
}
