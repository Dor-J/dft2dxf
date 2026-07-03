//! Core conversion errors.

use thiserror::Error;

/// Result alias for core operations.
pub type CoreResult<T> = Result<T, CoreError>;

/// Errors from in-memory conversion APIs.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CoreError {
  /// Input bytes are not a supported `.dft` format.
  #[error("unsupported or unrecognized .dft format")]
  UnsupportedFormat,

  /// Underlying reader/parser failure.
  #[error("{0}")]
  Read(String),

  /// Conversion or writer failure.
  #[error("{0}")]
  Convert(String),
}

impl CoreError {
  pub(crate) fn read(message: impl Into<String>) -> Self {
    Self::Read(message.into())
  }

  pub(crate) fn convert(message: impl Into<String>) -> Self {
    Self::Convert(message.into())
  }
}
