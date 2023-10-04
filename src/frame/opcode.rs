use crate::WSocketError;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum OpCode {
  Continuation = 0x0,
  Text = 0x1,
  Binary = 0x2,
  Close = 0x8,
  Ping = 0x9,
  Pong = 0xA,
}

impl TryFrom<u8> for OpCode {
  type Error = WSocketError;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0x0 => Ok(Self::Continuation),
      0x1 => Ok(Self::Text),
      0x2 => Ok(Self::Binary),
      0x8 => Ok(Self::Close),
      0x9 => Ok(Self::Ping),
      0xA => Ok(Self::Pong),
      code => Err(WSocketError::UnknownOpCode(code)),
    }
  }
}
