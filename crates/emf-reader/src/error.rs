//! EMF parser errors.

use thiserror::Error;

/// Result alias for EMF operations.
pub type EmfResult<T> = Result<T, EmfError>;

/// Errors while parsing or replaying EMF data.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum EmfError {
  /// Input is too short or truncated.
  #[error("invalid EMF at {context}: {message}")]
  InvalidFormat {
    /// Parser context.
    context: String,
    /// Detail message.
    message: String,
  },

  /// A configured limit was exceeded.
  #[error("limit exceeded: {kind} (limit {limit}, actual {actual})")]
  LimitExceeded {
    /// Limit kind.
    kind: &'static str,
    /// Configured limit.
    limit: u64,
    /// Observed value.
    actual: u64,
  },

  /// Record iterator reached EOF without `EMR_EOF`.
  #[error("EMF missing terminal EMR_EOF record")]
  MissingEof,
}

impl EmfError {
  pub(crate) fn invalid(context: impl Into<String>, message: impl Into<String>) -> Self {
    Self::InvalidFormat {
      context: context.into(),
      message: message.into(),
    }
  }

  pub(crate) fn limit(kind: &'static str, limit: u64, actual: u64) -> Self {
    Self::LimitExceeded {
      kind,
      limit,
      actual,
    }
  }
}
