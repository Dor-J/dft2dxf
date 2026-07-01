//! Errors for cncKad `.dft` parsing.

use std::path::PathBuf;

use thiserror::Error;

/// Result alias for cncKad reader operations.
pub type CkadResult<T> = Result<T, CkadError>;

/// Errors while reading cncKad `.dft` files.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CkadError {
  /// The input path could not be read.
  #[error("failed to read file {path}: {source}")]
  Io {
    /// File path.
    path: PathBuf,
    /// Underlying I/O error.
    #[source]
    source: std::io::Error,
  },

  /// The file exceeds a configured size limit.
  #[error("file exceeds size limit (limit {limit}, actual {actual})")]
  FileTooLarge {
    /// Configured limit in bytes.
    limit: u64,
    /// Observed size in bytes.
    actual: u64,
  },

  /// The file is not a cncKad `.dft` text drawing.
  #[error("not a cncKad .dft file: {message}")]
  NotCncKad {
    /// Human-readable detail.
    message: String,
  },

  /// Text content could not be parsed.
  #[error("invalid cncKad content at {context}: {message}")]
  InvalidFormat {
    /// Parser context.
    context: String,
    /// Human-readable detail.
    message: String,
  },
}
