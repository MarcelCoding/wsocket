use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use http_body_util::{Empty, Full};
use hyper::body::{Bytes, Incoming};
use hyper::client::conn::http1;
use hyper::header::{
  CONNECTION, HOST, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION, UPGRADE, USER_AGENT,
};
use hyper::StatusCode;
use hyper::{upgrade, Response};
use hyper::{Request, Uri};
use hyper_util::rt::tokio::TokioIo;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::error;

use crate::{UpgradedWebsocketIo, WSocketError, WebSocket};

pub async fn handshake<S>(
  uri: &Uri,
  host: &str,
  port: u16,
  user_agent: &str,
  socket: S,
  max_payload_len: usize,
  masking: bool,
) -> Result<(WebSocket<UpgradedWebsocketIo>, Response<Incoming>), WSocketError>
where
  S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
  let req = generate_request(uri, host, port, user_agent);

  let io = TokioIo::new(socket);

  let (mut sender, conn) = http1::handshake(io).await?;
  tokio::spawn(async move {
    if let Err(e) = conn.with_upgrades().await {
      error!("Error polling connection: {}", e);
    }
  });

  let mut response = sender.send_request(req).await?;
  verify(&response)?;

  let upgraded = upgrade::on(&mut response).await?;

  Ok((
    WebSocket::client(TokioIo::new(upgraded), max_payload_len, masking),
    response,
  ))
}

fn generate_request(uri: &Uri, host: &str, port: u16, user_agent: &str) -> Request<Empty<Bytes>> {
  let key: [u8; 16] = rand::random();
  let encoded_key = STANDARD.encode(key);

  Request::get(uri.to_string())
    .header(HOST, format!("{}:{}", host, port))
    .header(UPGRADE, "websocket")
    .header(CONNECTION, "upgrade")
    .header(SEC_WEBSOCKET_KEY, encoded_key)
    .header(SEC_WEBSOCKET_VERSION, "13")
    .header(USER_AGENT, user_agent)
    .body(Empty::new())
    .unwrap()
}

// https://github.com/snapview/tungstenite-rs/blob/314feea3055a93e585882fb769854a912a7e6dae/src/handshake/client.rs#L189
fn verify<B>(response: &Response<B>) -> Result<(), WSocketError> {
  if response.status() != StatusCode::SWITCHING_PROTOCOLS {
    return Err(WSocketError::InvalidStatusCode {
      actual: response.status(),
      expected: StatusCode::SWITCHING_PROTOCOLS,
    });
  }

  let headers = response.headers();

  if !headers
    .get(UPGRADE)
    .and_then(|h| h.to_str().ok())
    .map(|h| h.eq_ignore_ascii_case("websocket"))
    .unwrap_or(false)
  {
    return Err(WSocketError::InvalidUpgradeHeader);
  }

  if !headers
    .get(CONNECTION)
    .and_then(|h| h.to_str().ok())
    .map(|h| h.eq_ignore_ascii_case("upgrade"))
    .unwrap_or(false)
  {
    return Err(WSocketError::InvalidConnectionHeader);
  }

  Ok(())
}
