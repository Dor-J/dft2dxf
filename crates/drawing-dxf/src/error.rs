//! DXF writer errors.

use thiserror::Error;

/// Result alias for DXF writing.
pub type DxfResult<T> = Result<T, DxfError>;

/// Errors while writing DXF.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DxfError {
  /// DXF library failure.
  #[error("dxf write error: {0}")]
  Write(String),

  /// I/O failure.
  #[error("io error: {0}")]
  Io(#[from] std::io::Error),
}
