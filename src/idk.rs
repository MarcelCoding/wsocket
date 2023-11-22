use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use sha1::{Digest, Sha1};

pub(crate) fn sec_websocket_protocol(key: &[u8]) -> String {
  let mut sha1 = Sha1::default();
  sha1.update(key);
  sha1.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"); // magic string
  let result = sha1.finalize();
  STANDARD.encode(&result[..])
}
