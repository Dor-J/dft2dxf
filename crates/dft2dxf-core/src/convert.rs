//! In-memory `.dft` conversion for CLI and HTTP backends.

use ckad_reader::{detect_format, read_to_drawing, DftContainerFormat};
use dft_reader::{DftDocument, DftOpenOptions, Limits};
use drawing_dxf::{write_drawing_to_bytes, DxfWriteOptions};
use drawing_ir::{Drawing, DrawingMetadata, PaperUnit};
use drawing_svg::write_drawing_to_string;
use emf_reader::{
  replay_to_drawing, EmfDocument, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE,
};
use serde::Serialize;

use crate::error::{CoreError, CoreResult};

/// Options for [`convert_bytes`].
#[derive(Debug, Clone)]
pub struct ConvertOptions {
  /// Safety limits for hostile input.
  pub limits: Limits,
  /// One-based sheet index for Solid Edge files.
  pub sheet: Option<u32>,
  /// Optional units override (`mm`, `in`, `unitless`).
  pub units: Option<String>,
  /// Include SVG preview string in output.
  pub include_svg: bool,
  /// Include CAM/metadata JSON payload.
  pub include_cam_json: bool,
}

impl Default for ConvertOptions {
  fn default() -> Self {
    Self {
      limits: Limits::strict(),
      sheet: None,
      units: None,
      include_svg: false,
      include_cam_json: false,
    }
  }
}

/// Summary statistics for a conversion.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ConvertSummary {
  /// Detected source format (`cnckad` or `solid_edge`).
  pub format: String,
  /// Sheet index used for conversion.
  pub sheet: u32,
  /// Total entity count across sheets.
  pub entities: usize,
  /// Distinct layer count.
  pub layers: usize,
}

/// Output from [`convert_bytes`].
#[derive(Debug, Clone, Serialize)]
pub struct ConvertOutput {
  /// Conversion summary.
  pub summary: ConvertSummary,
  /// DXF file bytes.
  pub dxf: Vec<u8>,
  /// Optional SVG preview.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub svg: Option<String>,
  /// Optional CAM/metadata JSON.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cam_json: Option<serde_json::Value>,
  /// Diagnostics emitted during conversion.
  pub diagnostics: Vec<drawing_ir::Diagnostic>,
}

/// Loads a drawing from `.dft` bytes without writing output files.
///
/// # Errors
///
/// Returns [`CoreError`] when the format is unsupported or parsing fails.
pub fn load_drawing_from_bytes(input: &[u8], options: &ConvertOptions) -> CoreResult<Drawing> {
  let format = detect_format(input).ok_or(CoreError::UnsupportedFormat)?;
  let dir = tempfile::tempdir().map_err(|err| CoreError::read(err.to_string()))?;
  let path = dir.path().join("input.dft");
  std::fs::write(&path, input).map_err(|err| CoreError::read(err.to_string()))?;
  match format {
    DftContainerFormat::CncKad => read_to_drawing(&path, options.limits.max_file_size)
      .map_err(|err| CoreError::read(err.to_string())),
    DftContainerFormat::SolidEdgeCompound => load_solid_edge_drawing(&path, options),
  }
}

/// Converts `.dft` bytes to DXF (and optional SVG / CAM JSON).
///
/// # Errors
///
/// Returns [`CoreError`] when parsing or export fails.
pub fn convert_bytes(input: &[u8], options: &ConvertOptions) -> CoreResult<ConvertOutput> {
  let format = detect_format(input).ok_or(CoreError::UnsupportedFormat)?;
  let mut drawing = load_drawing_from_bytes(input, options)?;
  apply_units_override(&mut drawing, options.units.as_deref());

  let format_name = match format {
    DftContainerFormat::CncKad => "cnckad",
    DftContainerFormat::SolidEdgeCompound => "solid_edge",
  };

  let dxf = write_drawing_to_bytes(&mut drawing, DxfWriteOptions::default())
    .map_err(|err| CoreError::convert(err.to_string()))?;

  let svg = if options.include_svg {
    Some(
      write_drawing_to_string(&drawing).map_err(|err| CoreError::convert(err.to_string()))?,
    )
  } else {
    None
  };

  let cam_json = if options.include_cam_json {
    Some(build_cam_json(&drawing))
  } else {
    None
  };

  let sheet = options.sheet.unwrap_or(1);
  let summary = summarize_drawing(format_name, &drawing, sheet);
  let diagnostics = drawing.diagnostics.clone();

  Ok(ConvertOutput {
    summary,
    dxf,
    svg,
    cam_json,
    diagnostics,
  })
}

/// Applies a units override string to drawing metadata.
pub fn apply_units_override(drawing: &mut Drawing, units: Option<&str>) {
  let Some(units) = units else {
    return;
  };
  drawing.metadata.units = match units.to_ascii_lowercase().as_str() {
    "mm" | "millimeters" => PaperUnit::Millimeters,
    "in" | "inches" => PaperUnit::Inches,
    "unitless" => PaperUnit::Unitless,
    _ => drawing.metadata.units,
  };
}

/// Builds CAM/metadata JSON for API responses.
#[must_use]
pub fn build_cam_json(drawing: &Drawing) -> serde_json::Value {
  #[derive(Serialize)]
  struct Sidecar<'a> {
    metadata: &'a DrawingMetadata,
    cam: Option<&'a drawing_ir::CamProgram>,
  }
  serde_json::to_value(Sidecar {
    metadata: &drawing.metadata,
    cam: drawing.cam.as_ref(),
  })
  .unwrap_or(serde_json::json!({}))
}

fn load_solid_edge_drawing(path: &std::path::Path, options: &ConvertOptions) -> CoreResult<Drawing> {
  let mut document = DftDocument::open_with_options(path, &DftOpenOptions::new().with_limits(options.limits))
    .map_err(|err| CoreError::read(err.to_string()))?;

  let sheets = document
    .sheets()
    .map_err(|err| CoreError::read(err.to_string()))?;
  let index = options
    .sheet
    .unwrap_or_else(|| sheets.first().map_or(1, |sheet| sheet.index));
  let sheet_meta = document
    .sheet(index)
    .map_err(|err| CoreError::read(err.to_string()))?;
  let emf = document
    .extract_emf(index)
    .map_err(|err| CoreError::read(err.to_string()))?;

  let emf_doc = EmfDocument::parse(
    &emf.data,
    DEFAULT_MAX_RECORD_COUNT,
    DEFAULT_MAX_RECORD_SIZE,
  )
  .map_err(|err| CoreError::read(err.to_string()))?;

  Ok(replay_to_drawing(
    &emf_doc,
    Some(sheet_meta.index),
    Some(sheet_meta.name.clone()),
    Some(sheet_meta.info.width),
    Some(sheet_meta.info.height),
  ))
}

fn summarize_drawing(format: &str, drawing: &Drawing, sheet: u32) -> ConvertSummary {
  let entities: usize = drawing.sheets.iter().map(|s| s.entities.len()).sum();
  let mut layers = std::collections::BTreeSet::new();
  for sheet in &drawing.sheets {
    for entity in &sheet.entities {
      if let Some(layer) = &entity.layer {
        layers.insert(layer.clone());
      }
    }
  }
  ConvertSummary {
    format: format.to_string(),
    sheet,
    entities,
    layers: layers.len(),
  }
}
