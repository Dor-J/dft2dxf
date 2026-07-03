//! Validate `.dft` bytes.

use ckad_reader::{detect_format, DftContainerFormat};
use dft_reader::{DftDocument, DftOpenOptions};
use serde::Serialize;

use crate::convert::load_drawing_from_bytes;
use crate::error::{CoreError, CoreResult};
use crate::ConvertOptions;

/// Validation result.
#[derive(Debug, Clone, Serialize)]
pub struct ValidateOutput {
  /// Validation status.
  pub status: &'static str,
  /// Sheet count.
  pub sheets: usize,
}

/// Validates `.dft` bytes without producing conversion output.
///
/// # Errors
///
/// Returns [`CoreError`] when validation fails.
pub fn validate_bytes(input: &[u8], options: &ConvertOptions) -> CoreResult<ValidateOutput> {
  let format = detect_format(input).ok_or(CoreError::UnsupportedFormat)?;
  match format {
    DftContainerFormat::CncKad => {
      let drawing = load_drawing_from_bytes(input, options)?;
      Ok(ValidateOutput {
        status: "ok",
        sheets: drawing.sheets.len(),
      })
    }
    DftContainerFormat::SolidEdgeCompound => {
      let dir = tempfile::tempdir().map_err(|err| CoreError::read(err.to_string()))?;
      let path = dir.path().join("input.dft");
      std::fs::write(&path, input).map_err(|err| CoreError::read(err.to_string()))?;
      let mut document =
        DftDocument::open_with_options(&path, &DftOpenOptions::new().with_limits(options.limits))
          .map_err(|err| CoreError::read(err.to_string()))?;
      let report = document
        .inspect()
        .map_err(|err| CoreError::read(err.to_string()))?;
      if !report.storage.has_viewer_info || !report.storage.has_document_info {
        return Err(CoreError::read(
          "missing required JDraftViewerInfo metadata",
        ));
      }
      let sheets = document
        .sheets()
        .map_err(|err| CoreError::read(err.to_string()))?;
      for sheet in &sheets {
        document
          .extract_emf(sheet.index)
          .map_err(|err| CoreError::read(err.to_string()))?;
      }
      Ok(ValidateOutput {
        status: "ok",
        sheets: sheets.len(),
      })
    }
  }
}
