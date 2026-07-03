//! Inspect `.dft` bytes.

use ckad_reader::{detect_format, DftContainerFormat};
use dft_reader::{DftDocument, DftOpenOptions, InspectReport};
use serde::Serialize;

use crate::convert::load_drawing_from_bytes;
use crate::error::{CoreError, CoreResult};
use crate::ConvertOptions;

/// JSON-friendly inspect output.
#[derive(Debug, Clone, Serialize)]
pub struct InspectOutput {
  /// Detected format.
  pub format: String,
  /// Entity count when drawable.
  pub entities: usize,
  /// Sheet count when known.
  pub sheets: usize,
  /// Solid Edge inspect report when applicable.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub solid_edge: Option<InspectReport>,
}

/// Inspects `.dft` bytes and returns a summary.
///
/// # Errors
///
/// Returns [`CoreError`] when the input cannot be parsed.
pub fn inspect_bytes(input: &[u8], options: &ConvertOptions) -> CoreResult<InspectOutput> {
  let format = detect_format(input).ok_or(CoreError::UnsupportedFormat)?;
  match format {
    DftContainerFormat::CncKad => {
      let drawing = load_drawing_from_bytes(input, options)?;
      Ok(InspectOutput {
        format: "cnckad".to_string(),
        entities: drawing.sheets.iter().map(|s| s.entities.len()).sum(),
        sheets: drawing.sheets.len(),
        solid_edge: None,
      })
    }
    DftContainerFormat::SolidEdgeCompound => {
      let dir = tempfile::tempdir().map_err(|err| CoreError::read(err.to_string()))?;
      let path = dir.path().join("input.dft");
      std::fs::write(&path, input).map_err(|err| CoreError::read(err.to_string()))?;
      let mut document = DftDocument::open_with_options(
        &path,
        &DftOpenOptions::new().with_limits(options.limits),
      )
      .map_err(|err| CoreError::read(err.to_string()))?;
      let report = document
        .inspect()
        .map_err(|err| CoreError::read(err.to_string()))?;
      let sheets = document
        .sheets()
        .map_err(|err| CoreError::read(err.to_string()))?;
      Ok(InspectOutput {
        format: "solid_edge".to_string(),
        entities: 0,
        sheets: sheets.len(),
        solid_edge: Some(report),
      })
    }
  }
}
