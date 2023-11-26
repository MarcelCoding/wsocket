use std::io;
use std::string::FromUtf8Error;

#[cfg(all(feature = "handshake", feature = "client"))]
use hyper::StatusCode;
use thiserror::Error;

use crate::Close;
use crate::CloseCode;

pub type WSocketResult<T> = Result<T, WSocketError>;

#[derive(Error, Debug)]
pub enum WSocketError {
  #[error("unknown opcode `{0}`")]
  UnknownOpCode(u8),
  #[error("unknown close code `{0}`")]
  UnknownCloseCode(u16),
  #[error("reserve bit must be `0`")]
  ReserveBitMustBeNull,
  #[error("control frame must not be fragmented")]
  ControlFrameMustNotBeFragmented,
  #[error("control frame must have a payload length of 125 bytes or less")]
  ControlFrameMustHaveAPayloadLengthOf125BytesOrLess,
  #[error("payload too large")]
  PayloadTooLarge,
  #[error("io error")]
  Io(
    #[source]
    #[from]
    io::Error,
  ),
  #[error("not connected")]
  NotConnected,
  #[error("connection closed")]
  ConnectionClosed(Close),
  #[error("framed messages are not supported")]
  FramedMessagesAreNotSupported,
  #[error("text frames are not supported")]
  TextFramesAreNotSupported,
  #[error("invalid utf8")]
  InvalidUtf8(
    #[source]
    #[from]
    FromUtf8Error,
  ),
  #[error("invalid close close `{0}`")]
  InvalidCloseCode(u16),
  #[cfg(all(feature = "handshake", feature = "client"))]
  #[error("invalid status code `{actual}`, expected {expected}")]
  InvalidStatusCode {
    actual: StatusCode,
    expected: StatusCode,
  },
  #[error("invalid websocket http upgrade header")]
  InvalidUpgradeHeader,
  #[error("invalid websocket http connection header")]
  InvalidConnectionHeader,
  #[cfg(any(feature = "upgrade", all(feature = "client", feature = "handshake")))]
  #[error("hyper error")]
  Hyper(
    #[source]
    #[from]
    hyper::Error,
  ),
  #[error("missing sec web socket key")]
  MissingSecWebSocketKey,
  #[error("invalid sec websocket version")]
  InvalidSecWebsocketVersion,
}

impl WSocketError {
  pub(crate) fn is_io_error(&self) -> bool {
    matches!(self, WSocketError::Io(_))
  }

  pub(crate) fn close_code(&self) -> Option<CloseCode> {
    match self {
      Self::UnknownOpCode(_) => Some(CloseCode::ProtocolError),
      Self::UnknownCloseCode(_) => Some(CloseCode::ProtocolError),
      Self::ReserveBitMustBeNull => Some(CloseCode::Unsupported),
      Self::ControlFrameMustNotBeFragmented => Some(CloseCode::Unsupported),
      Self::ControlFrameMustHaveAPayloadLengthOf125BytesOrLess => Some(CloseCode::ProtocolError),
      Self::PayloadTooLarge => Some(CloseCode::MessageTooBig),
      Self::Io(_) => Some(CloseCode::Abnormal),
      Self::NotConnected => None,
      Self::ConnectionClosed(_) => None,
      Self::FramedMessagesAreNotSupported => Some(CloseCode::Unsupported),
      Self::TextFramesAreNotSupported => Some(CloseCode::Unsupported),
      Self::InvalidUtf8(_) => Some(CloseCode::InvalidPayload),
      Self::InvalidCloseCode(_) => None,
      #[cfg(all(feature = "handshake", feature = "client"))]
      Self::InvalidStatusCode { .. } => None,
      Self::InvalidUpgradeHeader => None,
      Self::InvalidConnectionHeader => None,
      #[cfg(any(feature = "upgrade", all(feature = "client", feature = "handshake")))]
      Self::Hyper(_) => None,
      Self::MissingSecWebSocketKey => None,
      Self::InvalidSecWebsocketVersion => None,
    }
  }
}
