//! `dft2dxf` HTTP conversion sidecar binary.

use std::net::SocketAddr;
use std::sync::Arc;

use dft2dxf_sidecar::{app, state::AppState};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();

  let state = Arc::new(AppState::from_env());
  let router = app(state);
  let host = std::env::var("SIDECAR_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
  let port = std::env::var("SIDECAR_PORT")
    .ok()
    .and_then(|value| value.parse().ok())
    .unwrap_or(8080);
  let addr: SocketAddr = format!("{host}:{port}")
    .parse()
    .expect("valid listen address");

  let listener = tokio::net::TcpListener::bind(addr)
    .await
    .expect("bind listen address");
  tracing::info!(%addr, "dft2dxf-sidecar listening");
  axum::serve(listener, router)
    .await
    .expect("server error");
}
