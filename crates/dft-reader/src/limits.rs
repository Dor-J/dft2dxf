//! Configurable safety limits for hostile `.dft` input.

use serde::Serialize;

/// Default maximum input file size (256 MiB).
pub const DEFAULT_MAX_FILE_SIZE: u64 = 256 * 1024 * 1024;

/// Default maximum compressed stream size (64 MiB).
pub const DEFAULT_MAX_STREAM_SIZE: u64 = 64 * 1024 * 1024;

/// Default maximum decompressed EMF size (256 MiB).
pub const DEFAULT_MAX_DECOMPRESSED_SIZE: u64 = 256 * 1024 * 1024;

/// Default maximum sheet count.
pub const DEFAULT_MAX_SHEET_COUNT: u32 = 1_024;

/// Default maximum CFB storage depth.
pub const DEFAULT_MAX_STORAGE_DEPTH: u32 = 32;

/// Default maximum CFB entry count.
pub const DEFAULT_MAX_ENTRY_COUNT: u32 = 100_000;

/// Safety limits applied while reading `.dft` files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Limits {
  /// Maximum `.dft` file size in bytes.
  pub max_file_size: u64,
  /// Maximum compressed stream size in bytes.
  pub max_stream_size: u64,
  /// Maximum decompressed EMF output size in bytes.
  pub max_decompressed_size: u64,
  /// Maximum number of sheets to enumerate.
  pub max_sheet_count: u32,
  /// Maximum CFB storage nesting depth.
  pub max_storage_depth: u32,
  /// Maximum total CFB entries while walking the tree.
  pub max_entry_count: u32,
}

impl Default for Limits {
  fn default() -> Self {
    Self::strict()
  }
}

impl Limits {
  /// Returns conservative production defaults.
  #[must_use]
  pub const fn strict() -> Self {
    Self {
      max_file_size: DEFAULT_MAX_FILE_SIZE,
      max_stream_size: DEFAULT_MAX_STREAM_SIZE,
      max_decompressed_size: DEFAULT_MAX_DECOMPRESSED_SIZE,
      max_sheet_count: DEFAULT_MAX_SHEET_COUNT,
      max_storage_depth: DEFAULT_MAX_STORAGE_DEPTH,
      max_entry_count: DEFAULT_MAX_ENTRY_COUNT,
    }
  }

  /// Returns relaxed limits for trusted local debugging.
  #[must_use]
  pub const fn relaxed() -> Self {
    Self {
      max_file_size: 512 * 1024 * 1024,
      max_stream_size: 128 * 1024 * 1024,
      max_decompressed_size: 512 * 1024 * 1024,
      max_sheet_count: 2_048,
      max_storage_depth: 64,
      max_entry_count: 200_000,
    }
  }
}
