//! Sheet metadata and extracted EMF payload.

use std::io::Write;
use std::path::Path;

use serde::Serialize;

use crate::error::{DftError, DftResult};
use crate::metadata::SheetInfo;

/// One draft sheet discovered in viewer metadata.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sheet {
  /// One-based sheet index used for stream names.
  pub index: u32,
  /// Sheet display name from metadata.
  pub name: String,
  /// Parsed sheet info block.
  pub info: SheetInfo,
}

impl Sheet {
  /// Returns the CFB stream name for this sheet (`"1"`, `"2"`, ...).
  #[must_use]
  pub fn stream_name(&self) -> String {
    self.index.to_string()
  }
}

/// Decompressed EMF bytes for one sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedEmf {
  /// One-based sheet index.
  pub sheet_index: u32,
  /// Raw EMF bytes.
  pub data: Vec<u8>,
}

impl ExtractedEmf {
  /// Creates a new extracted EMF payload.
  #[must_use]
  pub fn new(sheet_index: u32, data: Vec<u8>) -> Self {
    Self { sheet_index, data }
  }

  /// Writes EMF bytes to `path`.
  ///
  /// # Errors
  ///
  /// Returns [`DftError::Io`] if the parent directory or output file cannot be created or written.
  pub fn write_to(&self, path: &Path) -> DftResult<()> {
    if let Some(parent) = path.parent() {
      if !parent.as_os_str().is_empty() {
        std::fs::create_dir_all(parent).map_err(|source| DftError::Io {
          path: parent.to_path_buf(),
          source,
        })?;
      }
    }
    let mut file = std::fs::File::create(path).map_err(|source| DftError::Io {
      path: path.to_path_buf(),
      source,
    })?;
    file.write_all(&self.data).map_err(|source| DftError::Io {
      path: path.to_path_buf(),
      source,
    })?;
    Ok(())
  }
}
