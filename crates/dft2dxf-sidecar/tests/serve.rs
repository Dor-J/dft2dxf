//! TCP-level sidecar server tests.

use std::sync::Arc;
use std::time::Duration;

use dft2dxf_sidecar::{listen_addr, serve_listener, state::AppState};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn http_get(addr: std::net::SocketAddr, path: &str) -> String {
  let mut stream = TcpStream::connect(addr).await.unwrap();
  let request = format!(
    "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
  );
  stream.write_all(request.as_bytes()).await.unwrap();
  let mut response = String::new();
  stream.read_to_string(&mut response).await.unwrap();
  response
}

#[test]
fn listen_addr_parses_host_and_port() {
  let addr = listen_addr("127.0.0.1", 9090);
  assert_eq!(addr.ip().to_string(), "127.0.0.1");
  assert_eq!(addr.port(), 9090);
}

#[tokio::test]
async fn serve_listener_responds_to_health_over_tcp() {
  let state = Arc::new(AppState::with_concurrency(2));
  let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let addr = listener.local_addr().unwrap();

  let handle = tokio::spawn(async move { serve_listener(listener, state).await });

  tokio::time::sleep(Duration::from_millis(100)).await;
  let response = http_get(addr, "/health").await;
  assert!(response.contains("200"));
  assert!(response.contains("\"status\":\"ok\""));

  handle.abort();
}

#[tokio::test]
async fn serve_listener_ready_reports_available_workers() {
  let state = Arc::new(AppState::with_concurrency(3));
  let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
  let addr = listener.local_addr().unwrap();

  let handle = tokio::spawn(async move { serve_listener(listener, state).await });

  tokio::time::sleep(Duration::from_millis(100)).await;
  let response = http_get(addr, "/ready").await;
  assert!(response.contains("200"));
  assert!(response.contains("\"available_workers\":3"));

  handle.abort();
}
