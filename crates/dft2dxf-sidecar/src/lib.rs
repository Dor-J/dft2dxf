//! HTTP sidecar library surface for tests and embedding.

pub mod routes;
pub mod state;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
  routing::{get, post},
  Router,
};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::routes::{convert, health, inspect, ready, validate};
use crate::state::AppState;

/// Builds the Axum router for tests and production.
pub fn app(state: Arc<AppState>) -> Router {
  Router::new()
    .route("/health", get(health))
    .route("/ready", get(ready))
    .route("/v1/convert", post(convert))
    .route("/v1/inspect", post(inspect))
    .route("/v1/validate", post(validate))
    .layer(TraceLayer::new_for_http())
    .with_state(state)
}

/// Parses a `host:port` listen address.
///
/// # Panics
///
/// Panics when `host` or `port` do not form a valid socket address.
#[must_use]
pub fn listen_addr(host: &str, port: u16) -> SocketAddr {
  format!("{host}:{port}")
    .parse()
    .expect("valid listen address")
}

/// Resolves the listen address from `SIDECAR_HOST` and `SIDECAR_PORT`.
#[must_use]
pub fn listen_addr_from_env() -> SocketAddr {
  let host = std::env::var("SIDECAR_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
  let port = std::env::var("SIDECAR_PORT")
    .ok()
    .and_then(|value| value.parse().ok())
    .unwrap_or(8080);
  listen_addr(&host, port)
}

/// Serves the sidecar on an already-bound listener until shutdown.
///
/// # Errors
///
/// Returns an I/O error when the HTTP server fails.
pub async fn serve_listener(listener: TcpListener, state: Arc<AppState>) -> std::io::Result<()> {
  let addr = listener.local_addr()?;
  tracing::info!(%addr, "dft2dxf-sidecar listening");
  axum::serve(listener, app(state)).await
}

/// Binds `addr` and serves the sidecar until shutdown.
///
/// # Errors
///
/// Returns an I/O error when binding or serving fails.
pub async fn serve(addr: SocketAddr, state: Arc<AppState>) -> std::io::Result<()> {
  serve_listener(TcpListener::bind(addr).await?, state).await
}

/// Starts the sidecar using environment-derived listen settings.
///
/// # Errors
///
/// Returns an I/O error when binding or serving fails.
pub async fn run(state: Arc<AppState>) -> std::io::Result<()> {
  serve(listen_addr_from_env(), state).await
}

/// Initializes tracing from `RUST_LOG` / `RUST_LOG_STYLE` (idempotent).
pub fn init_tracing() {
  let _ = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .try_init();
}

/// Initializes tracing and starts the sidecar from environment variables.
///
/// # Errors
///
/// Returns an I/O error when binding or serving fails.
pub async fn run_from_env() -> std::io::Result<()> {
  init_tracing();
  run(Arc::new(AppState::from_env())).await
}

#[cfg(test)]
mod tests {
  use super::init_tracing;

  #[test]
  fn init_tracing_can_be_called_more_than_once() {
    init_tracing();
    init_tracing();
  }
}
