//! Shared HTTP application state.

use std::sync::Arc;

use dft_reader::Limits;
use tokio::sync::Semaphore;

/// Application state shared across HTTP handlers.
pub struct AppState {
  /// Limits concurrent CPU-bound conversions.
  pub pool: Arc<Semaphore>,
  /// Parser safety limits.
  pub limits: Limits,
}

impl AppState {
  /// Creates state from environment variables.
  #[must_use]
  pub fn from_env() -> Self {
    let concurrency = std::env::var("WORKER_CONCURRENCY")
      .ok()
      .and_then(|value| value.parse().ok())
      .unwrap_or_else(|| std::thread::available_parallelism().map_or(2, std::num::NonZero::get));
    Self {
      pool: Arc::new(Semaphore::new(concurrency)),
      limits: Limits::strict(),
    }
  }
}
