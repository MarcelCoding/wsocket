use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::{CONNECTION, UPGRADE};
use hyper::upgrade::Upgraded;
use hyper::Request;
use hyper::Response;
use hyper_util::rt::TokioIo;
use pin_project_lite::pin_project;

use crate::idk::sec_websocket_protocol;
use crate::{WSocketError, WebSocket};

pin_project! {
  pub struct UpgradeFut {
    #[pin]
    inner: hyper::upgrade::OnUpgrade,
    max_payload_len: usize,
  }
}

pub fn upgrade<B>(
  mut request: impl std::borrow::BorrowMut<Request<B>>,
  max_payload_len: usize,
) -> Result<(Response<Full<Bytes>>, UpgradeFut), WSocketError> {
  let request = request.borrow_mut();

  let key = request
    .headers()
    .get("Sec-WebSocket-Key")
    .ok_or(WSocketError::MissingSecWebSocketKey)?;
  if request
    .headers()
    .get("Sec-WebSocket-Version")
    .map(|v| v.as_bytes())
    != Some(b"13")
  {
    return Err(WSocketError::InvalidSecWebsocketVersion);
  }

  let response = Response::builder()
    .status(hyper::StatusCode::SWITCHING_PROTOCOLS)
    .header(CONNECTION, "upgrade")
    .header(UPGRADE, "websocket")
    .header(
      "Sec-WebSocket-Accept",
      &sec_websocket_protocol(key.as_bytes()),
    )
    .body(Full::new(Bytes::from("switching to websocket protocol")))
    .expect("bug: failed to build response");

  let stream = UpgradeFut {
    inner: hyper::upgrade::on(request),
    max_payload_len,
  };

  Ok((response, stream))
}

pub fn is_upgrade_request<B>(request: &Request<B>) -> bool {
  header_contains_value(request.headers(), CONNECTION, "Upgrade")
    && header_contains_value(request.headers(), UPGRADE, "websocket")
}

fn header_contains_value(
  headers: &hyper::HeaderMap,
  header: impl hyper::header::AsHeaderName,
  value: impl AsRef<[u8]>,
) -> bool {
  let value = value.as_ref();
  for header in headers.get_all(header) {
    if header
      .as_bytes()
      .split(|&c| c == b',')
      .any(|x| trim(x).eq_ignore_ascii_case(value))
    {
      return true;
    }
  }
  false
}

fn trim(data: &[u8]) -> &[u8] {
  trim_end(trim_start(data))
}

fn trim_start(data: &[u8]) -> &[u8] {
  if let Some(start) = data.iter().position(|x| !x.is_ascii_whitespace()) {
    &data[start..]
  } else {
    b""
  }
}

fn trim_end(data: &[u8]) -> &[u8] {
  if let Some(last) = data.iter().rposition(|x| !x.is_ascii_whitespace()) {
    &data[..last + 1]
  } else {
    b""
  }
}

impl std::future::Future for UpgradeFut {
  type Output = Result<WebSocket<TokioIo<Upgraded>>, WSocketError>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.project();
    let upgraded = match this.inner.poll(cx) {
      Poll::Pending => return Poll::Pending,
      Poll::Ready(Ok(x)) => x,
      Poll::Ready(Err(err)) => {
        return Poll::Ready(Err(err.into()));
      }
    };

    let io = TokioIo::new(upgraded);

    Poll::Ready(Ok(WebSocket::server(io, *this.max_payload_len)))
  }
}
