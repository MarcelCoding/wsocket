use std::io::Cursor;

use crate::frame::{Frame, OpCode};
use crate::WSocketResult;

macro_rules! test_read_frame {
  ($($name:ident: ($input:expr, $size:expr, $fin:expr, $opcode:expr, $data:expr),)*) => {
    $(
      #[tokio::test]
      async fn $name() -> WSocketResult<()> {
        let mut buf = [0u8; $size];

        let mut read = Cursor::new($input);
        let frame = Frame::read(&mut read, &mut buf, $size).await?;

        assert_eq!(frame.fin, $fin);
        assert_eq!(frame.opcode, $opcode);
        assert_eq!(frame.data, $data);

        Ok(())
      }
    )*
  }
}

// tests taken from https://datatracker.ietf.org/doc/html/rfc6455#section-5.7
test_read_frame! {
  test_read_unmasked_frame: (
    [0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f],
    5,
    true,
    OpCode::Text,
    "Hello".as_bytes()
  ),
  test_read_masked_frame: (
    [0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58],
    5,
    true,
    OpCode::Text,
    "Hello".as_bytes()
  ),
  test_read_unmasked_fragmented_frame: (
    [0x01, 0x03, 0x48, 0x65, 0x6c],
    3,
    false,
    OpCode::Text,
    "Hel".as_bytes()
  ),
  test_read_unmasked_fragmented_fin_frame: (
    [0x80, 0x02, 0x6c, 0x6f],
    2,
    true,
    OpCode::Continuation,
    "lo".as_bytes()
  ),
  test_read_unmasked_ping_frame: (
    [0x89, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f],
    5,
    true,
    OpCode::Ping,
    "Hello".as_bytes()
  ),
  test_read_masked_pong_frame: (
    [0x8a, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58],
    5,
    true,
    OpCode::Pong,
    "Hello".as_bytes()
  ),
  test_read_256_binary_unmasked_frame: (
    include_bytes!("../test/frame_256_in.bin"),
    256,
    true,
    OpCode::Binary,
    include_bytes!("../test/frame_256_out.bin")
  ),
  test_read_65kib_binary_unmasked_frame: (
    include_bytes!("../test/frame_65536_in.bin"),
    65536,
    true,
    OpCode::Binary,
    include_bytes!("../test/frame_65536_out.bin")
  ),
}
