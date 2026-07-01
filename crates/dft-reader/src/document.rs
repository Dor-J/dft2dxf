//! High-level `.dft` document API.

use std::path::{Path, PathBuf};

use cfb::CompoundFile;
use serde::Serialize;

use crate::error::{DftError, DftResult};
use crate::limits::Limits;
use crate::metadata::{
  DraftDocumentInfo, StorageTree, STORAGE_J_DRAFT_VIEWER_INFO, STREAM_J_DRAFT_DOCUMENT_INFO,
};
use crate::sheet::{ExtractedEmf, Sheet};
use crate::storage::{
  build_storage_tree, extract_sheet_emf, open_compound_file, parse_draft_metadata,
  read_stream_limited, ParsedDraft,
};

/// Options used when opening a `.dft` file.
#[derive(Debug, Clone)]
pub struct DftOpenOptions {
  limits: Limits,
}

impl Default for DftOpenOptions {
  fn default() -> Self {
    Self {
      limits: Limits::strict(),
    }
  }
}

impl DftOpenOptions {
  /// Creates options with default strict limits.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Overrides safety limits.
  #[must_use]
  pub fn with_limits(mut self, limits: Limits) -> Self {
    self.limits = limits;
    self
  }

  /// Returns configured limits.
  #[must_use]
  pub fn limits(&self) -> Limits {
    self.limits
  }
}

/// Inspection report for a `.dft` file.
#[derive(Debug, Clone, Serialize)]
pub struct InspectReport {
  /// Source file path.
  pub path: PathBuf,
  /// Compound file storage tree.
  pub storage: StorageTree,
  /// Parsed draft metadata when available.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub document_info: Option<DraftDocumentInfo>,
  /// Parsed sheets when metadata is available.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub sheets: Vec<Sheet>,
}

/// An opened Solid Edge draft file.
pub struct DftDocument {
  path: PathBuf,
  compound: CompoundFile<std::fs::File>,
  limits: Limits,
  parsed: Option<ParsedDraft>,
}

impl DftDocument {
  /// Opens a `.dft` file from disk.
  pub fn open(path: impl AsRef<Path>) -> DftResult<Self> {
    Self::open_with_options(path, DftOpenOptions::default())
  }

  /// Opens a `.dft` file using custom options.
  pub fn open_with_options(path: impl AsRef<Path>, options: DftOpenOptions) -> DftResult<Self> {
    let path = path.as_ref().to_path_buf();
    let limits = options.limits();
    let compound = open_compound_file(&path, &limits)?;
    Ok(Self {
      path,
      compound,
      limits,
      parsed: None,
    })
  }

  /// Returns the source path.
  #[must_use]
  pub fn path(&self) -> &Path {
    &self.path
  }

  /// Returns configured limits.
  #[must_use]
  pub fn limits(&self) -> Limits {
    self.limits
  }

  /// Builds an inspection report without extracting EMF payloads.
  pub fn inspect(&mut self) -> DftResult<InspectReport> {
    let path = self.path.clone();
    let storage = build_storage_tree(&mut self.compound, &self.limits)?;
    let parsed = self.load_parsed().ok();
    Ok(InspectReport {
      path,
      storage,
      document_info: parsed.as_ref().map(|value| value.document_info.clone()),
      sheets: parsed
        .map(|value| value.sheets.clone())
        .unwrap_or_default(),
    })
  }

  /// Returns all sheets declared in viewer metadata.
  pub fn sheets(&mut self) -> DftResult<Vec<Sheet>> {
    Ok(self.load_parsed()?.sheets.clone())
  }

  /// Returns one sheet by one-based index.
  pub fn sheet(&mut self, one_based_index: u32) -> DftResult<Sheet> {
    let parsed = self.load_parsed()?;
    parsed
      .sheets
      .iter()
      .find(|sheet| sheet.index == one_based_index)
      .cloned()
      .ok_or(DftError::SheetOutOfRange {
        index: one_based_index,
        max: parsed.sheets.len() as u32,
      })
  }

  /// Extracts the EMF payload for one sheet (one-based index).
  pub fn extract_emf(&mut self, one_based_index: u32) -> DftResult<ExtractedEmf> {
    let sheet = self.sheet(one_based_index)?;
    let data = extract_sheet_emf(&mut self.compound, &sheet, &self.limits)?;
    Ok(ExtractedEmf::new(sheet.index, data))
  }

  fn load_parsed(&mut self) -> DftResult<&ParsedDraft> {
    if self.parsed.is_none() {
      let metadata_path = format!("{STORAGE_J_DRAFT_VIEWER_INFO}/{STREAM_J_DRAFT_DOCUMENT_INFO}");
      let data = read_stream_limited(&mut self.compound, &metadata_path, &self.limits)?;
      let parsed = parse_draft_metadata(&data, &self.limits)?;
      self.parsed = Some(parsed);
    }
    Ok(self.parsed.as_ref().expect("parsed draft metadata"))
  }
}
