//! Limits and storage tests for dft-reader.

use dft2dxf_testkit::{build_minimal_dft, build_rectangle_emf, MinimalDftSpec};
use dft_reader::{DftDocument, DftOpenOptions, Limits};

#[test]
fn strict_and_relaxed_limits_differ() {
  let strict = Limits::strict();
  let relaxed = Limits::relaxed();
  assert!(relaxed.max_file_size > strict.max_file_size);
  assert!(relaxed.max_storage_depth > strict.max_storage_depth);
}

#[test]
fn open_options_applies_custom_limits() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 10, 10);
  build_minimal_dft(&path, &MinimalDftSpec::one_sheet("S1", emf)).unwrap();

  let mut limits = Limits::strict();
  limits.max_file_size = 1;
  let result = DftDocument::open_with_options(&path, DftOpenOptions::new().with_limits(limits));
  assert!(result.is_err());
}

#[test]
fn inspect_lists_storage_entries() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 10, 10);
  build_minimal_dft(&path, &MinimalDftSpec::one_sheet("S1", emf)).unwrap();

  let mut document = DftDocument::open(&path).unwrap();
  let report = document.inspect().unwrap();
  assert!(!report.storage.entries.is_empty());
  assert!(report.storage.has_viewer_info);
}

#[test]
fn document_sheet_lookup_and_metadata() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 10, 10);
  build_minimal_dft(&path, &MinimalDftSpec::one_sheet("Named", emf)).unwrap();

  let mut document = DftDocument::open(&path).unwrap();
  let sheet = document.sheet(1).unwrap();
  assert_eq!(sheet.name, "Named");
  let sheets = document.sheets().unwrap();
  assert_eq!(sheets.len(), 1);
}
