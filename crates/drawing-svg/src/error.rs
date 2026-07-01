//! SVG writer errors.

use thiserror::Error;

/// Result alias for SVG writing.
pub type SvgResult<T> = Result<T, SvgError>;

/// Errors while writing SVG.
#[derive(Debug, Error)]
pub enum SvgError {
  /// I/O failure.
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
}
