//! Draft document and sheet metadata structures.

use serde::Serialize;

/// Known storage name for embedded draft viewer data.
pub const STORAGE_J_DRAFT_VIEWER_INFO: &str = "JDraftViewerInfo";

/// Known stream name for draft document metadata inside viewer storage.
pub const STREAM_J_DRAFT_DOCUMENT_INFO: &str = "JDraftDocumentInfo";

/// EMF file signature (`EMF ` little-endian).
pub const EMF_SIGNATURE: u32 = 0x0000_464D;

/// Paper units used by Solid Edge draft sheets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaperUnit {
  /// Unknown unit value.
  Unknown(u16),
  /// Millimeters.
  Millimeters,
  /// Centimeters.
  Centimeters,
  /// Inches.
  Inches,
}

impl PaperUnit {
  /// Parses a raw unit field from draft metadata.
  #[must_use]
  pub fn from_raw(value: u16) -> Self {
    match value {
      0x003D => Self::Millimeters,
      0x003E => Self::Centimeters,
      0x0040 => Self::Inches,
      other => Self::Unknown(other),
    }
  }
}

/// Parsed `JDraftDocumentInfo` header.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DraftDocumentInfo {
  /// Viewer info format version.
  pub viewer_info_version: u32,
  /// Number of sheets declared in metadata.
  pub number_of_sheets: i32,
  /// Active sheet index from metadata.
  pub active_sheet_index: i32,
  /// Geometric version field from metadata.
  pub geometric_version: u32,
  /// Paper units for sheet dimensions.
  pub units: PaperUnit,
}

/// Per-sheet metadata from `JDraftDocumentInfo`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SheetInfo {
  /// Sheet width in paper units.
  pub width: f64,
  /// Sheet height in paper units.
  pub height: f64,
  /// Declared uncompressed EMF size in bytes.
  pub emf_size: u32,
  /// Declared compressed EMF size in bytes.
  pub emf_compressed_size: u32,
}

/// One node in a compound file storage tree.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StorageEntry {
  /// Entry path using `/` separators.
  pub path: String,
  /// Entry kind.
  pub kind: StorageEntryKind,
  /// Stream size in bytes when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub size: Option<u64>,
}

/// Compound file entry kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageEntryKind {
  /// Storage (directory) entry.
  Storage,
  /// Stream (file) entry.
  Stream,
}

/// Full storage tree summary for inspection output.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StorageTree {
  /// Flat list of entries in deterministic order.
  pub entries: Vec<StorageEntry>,
  /// Whether `JDraftViewerInfo` exists.
  pub has_viewer_info: bool,
  /// Whether `JDraftViewerInfo/JDraftDocumentInfo` exists.
  pub has_document_info: bool,
}
