//! HTTP sidecar library surface for tests and embedding.

pub mod routes;
pub mod state;

use std::sync::Arc;

use axum::{
  routing::{get, post},
  Router,
};
use tower_http::trace::TraceLayer;

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
