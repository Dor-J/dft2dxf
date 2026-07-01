//! Typed errors for `.dft` reading.

use std::path::PathBuf;

use thiserror::Error;

/// Result alias for `dft-reader` operations.
pub type DftResult<T> = Result<T, DftError>;

/// Errors that can occur while reading or extracting `.dft` files.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DftError {
  /// The input path could not be read.
  #[error("failed to read file {path}: {source}")]
  Io {
    /// File path.
    path: PathBuf,
    /// Underlying I/O error.
    #[source]
    source: std::io::Error,
  },

  /// The file is not a valid compound file.
  #[error("not a compound file: {message}")]
  NotCompoundFile {
    /// Human-readable detail.
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

  /// Required Solid Edge viewer storage was not found.
  #[error("missing required storage or stream: {path}")]
  MissingViewerData {
    /// Storage/stream path.
    path: String,
  },

  /// Binary metadata could not be parsed.
  #[error("invalid draft metadata at {context}: {message}")]
  InvalidMetadata {
    /// Parser context.
    context: String,
    /// Human-readable detail.
    message: String,
  },

  /// Decompression failed or produced invalid output.
  #[error("decompression failed for {stream}: {message}")]
  DecompressionFailed {
    /// Stream name.
    stream: String,
    /// Human-readable detail.
    message: String,
  },

  /// Extracted bytes do not look like a valid EMF.
  #[error("invalid EMF payload for sheet {sheet_index}: {message}")]
  InvalidEmf {
    /// One-based sheet index.
    sheet_index: u32,
    /// Human-readable detail.
    message: String,
  },

  /// Sheet index is out of range.
  #[error("sheet index {index} out of range (1..={max})")]
  SheetOutOfRange {
    /// Requested one-based sheet index.
    index: u32,
    /// Maximum valid one-based sheet index.
    max: u32,
  },

  /// Low-level CFB error.
  #[error("compound file error: {0}")]
  CompoundFile(#[from] cfb::Error),
}

impl DftError {
  /// Builds a limit-exceeded error.
  pub(crate) fn limit(kind: &'static str, limit: u64, actual: u64) -> Self {
    Self::LimitExceeded { kind, limit, actual }
  }
}
