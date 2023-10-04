use std::fmt::{Display, Formatter};

use crate::{WSocketError, WSocketResult};

/// When closing an established connection an endpoint MAY indicate a reason for closure.
/// <https://datatracker.ietf.org/doc/html/rfc6455#section-7.4.1>
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum CloseCode {
  /// The purpose for which the connection was established has been fulfilled.
  Normal = 1000,
  /// Server going down or a browser having navigated away from a page.
  Away = 1001,
  /// An endpoint is terminating the connection due to a protocol error.
  ProtocolError = 1002,
  /// It has received a type of data it cannot accept.
  Unsupported = 1003,
  /// No status code was actually present.
  NoStatusRcvd = 1005,
  /// Connection was closed abnormally.
  Abnormal = 1006,
  /// Application has received data within a message that was not consistent with the type of the message.
  InvalidPayload = 1007,
  /// This is a generic status code that can be returned when there is no other more suitable status code.
  PolicyViolation = 1008,
  /// Message that is too big for it to process.
  MessageTooBig = 1009,
  /// The client has expected the server to negotiate one or more extension.
  MandatoryExt = 1010,
  /// The server has encountered an unexpected condition that prevented it from fulfilling the request.
  InternalError = 1011,
  /// The connection was closed due to a failure to perform a TLS handshake.
  TlsHandshake = 1015,
}

impl CloseCode {
  /// Whether it is allowed to send this status code in a close frame.
  pub fn is_send_allowed(&self) -> bool {
    match self {
      CloseCode::Normal => true,
      CloseCode::Away => true,
      CloseCode::ProtocolError => true,
      CloseCode::Unsupported => false,
      CloseCode::NoStatusRcvd => false,
      CloseCode::Abnormal => false,
      CloseCode::InvalidPayload => true,
      CloseCode::PolicyViolation => true,
      CloseCode::MessageTooBig => true,
      CloseCode::MandatoryExt => true,
      CloseCode::InternalError => true,
      CloseCode::TlsHandshake => false,
    }
  }
}

impl TryFrom<u16> for CloseCode {
  type Error = WSocketError;

  #[inline]
  fn try_from(value: u16) -> Result<Self, Self::Error> {
    match value {
      1000 => Ok(Self::Normal),
      1001 => Ok(Self::Away),
      1002 => Ok(Self::ProtocolError),
      1003 => Ok(Self::Unsupported),
      1005 => Ok(Self::NoStatusRcvd),
      1006 => Ok(Self::Abnormal),
      1007 => Ok(Self::InvalidPayload),
      1009 => Ok(Self::MessageTooBig),
      1010 => Ok(Self::MandatoryExt),
      1011 => Ok(Self::InternalError),
      1015 => Ok(Self::TlsHandshake),
      code => Err(WSocketError::UnknownCloseCode(code)),
    }
  }
}

impl Display for CloseCode {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      CloseCode::Normal => write!(f, "Normal"),
      CloseCode::Away => write!(f, "Normal"),
      CloseCode::ProtocolError => write!(f, "ProtocolError"),
      CloseCode::Unsupported => write!(f, "Unsupported"),
      CloseCode::NoStatusRcvd => write!(f, "NoStatusRcvd"),
      CloseCode::Abnormal => write!(f, "Abnormal"),
      CloseCode::InvalidPayload => write!(f, "InvalidPayload"),
      CloseCode::PolicyViolation => write!(f, "PolicyViolation"),
      CloseCode::MessageTooBig => write!(f, "MessageTooBig"),
      CloseCode::MandatoryExt => write!(f, "MandatoryExt"),
      CloseCode::InternalError => write!(f, "InternalError"),
      CloseCode::TlsHandshake => write!(f, "TlsHandshake"),
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Close {
  code: CloseCode,
  reason: Option<String>,
}

impl Close {
  pub fn new(code: CloseCode, reason: Option<String>) -> Self {
    Self { code, reason }
  }

  pub(crate) fn encode(&self) -> WSocketResult<Vec<u8>> {
    if !self.code.is_send_allowed() {
      todo!("Not implemented");
    }

    if let Some(reason) = &self.reason {
      let mut buf = Vec::with_capacity(2 + reason.len());
      buf.copy_from_slice(&(self.code as u16).to_be_bytes());
      buf.copy_from_slice(reason.as_bytes());
      Ok(buf)
    } else {
      let mut buf = Vec::with_capacity(2);
      buf.copy_from_slice(&(self.code as u16).to_be_bytes());
      Ok(buf)
    }
  }

  pub(crate) fn parse(payload: &[u8]) -> WSocketResult<Self> {
    let len = payload.len();

    let code = if len >= 2 {
      let b1 = *unsafe { payload.get_unchecked(0) };
      let b2 = *unsafe { payload.get_unchecked(1) };
      let raw_code = [b1, b2];
      let raw_code = u16::from_be_bytes(raw_code);
      CloseCode::try_from(raw_code)?
    } else {
      CloseCode::NoStatusRcvd
    };

    let reason = if len >= 3 {
      let buf = unsafe { payload.get_unchecked(2..) };
      Some(String::from_utf8(buf.to_vec())?)
    } else {
      None
    };

    Ok(Self { code, reason })
  }
}
