//! Metadata parser tests for Solid Edge `.dft` viewer info.

use std::path::PathBuf;

use dft2dxf_testkit::{
  excessive_sheet_count_metadata, negative_sheet_count_metadata, too_short_metadata,
};
use dft_reader::{parse_viewer_document_info, DftError, Limits};

fn fixture_path(name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../../tests/fixtures/malformed")
    .join(name)
}

#[test]
fn rejects_truncated_metadata_bytes() {
  let data = std::fs::read(fixture_path("too-short-metadata.bin")).unwrap();
  let err = parse_viewer_document_info(&data, &Limits::strict()).unwrap_err();
  assert!(matches!(err, DftError::InvalidMetadata { .. }));

  let synthetic = too_short_metadata();
  let err = parse_viewer_document_info(&synthetic, &Limits::strict()).unwrap_err();
  assert!(matches!(err, DftError::InvalidMetadata { .. }));
}

#[test]
fn rejects_negative_sheet_count() {
  let data = std::fs::read(fixture_path("negative-sheet-count.bin")).unwrap();
  let err = parse_viewer_document_info(&data, &Limits::strict()).unwrap_err();
  assert!(matches!(err, DftError::InvalidMetadata { .. }));

  let synthetic = negative_sheet_count_metadata();
  let err = parse_viewer_document_info(&synthetic, &Limits::strict()).unwrap_err();
  assert!(matches!(err, DftError::InvalidMetadata { .. }));
}

#[test]
fn rejects_excessive_sheet_count() {
  let limits = Limits {
    max_sheet_count: 2,
    ..Limits::strict()
  };
  let data = excessive_sheet_count_metadata(3);
  let err = parse_viewer_document_info(&data, &limits).unwrap_err();
  assert!(matches!(err, DftError::LimitExceeded { .. }));
}

#[test]
fn parses_valid_metadata_from_synthetic_dft() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("sample.dft");
  let emf = dft2dxf_testkit::build_rectangle_emf(0, 0, 10, 10);
  dft2dxf_testkit::build_minimal_dft(&path, &dft2dxf_testkit::MinimalDftSpec::one_sheet("A", emf))
    .unwrap();

  let mut compound = cfb::open(&path).unwrap();
  let mut stream = compound
    .open_stream("JDraftViewerInfo/JDraftDocumentInfo")
    .unwrap();
  let mut data = Vec::new();
  std::io::Read::read_to_end(&mut stream, &mut data).unwrap();

  let (info, sheets) = parse_viewer_document_info(&data, &Limits::strict()).unwrap();
  assert_eq!(info.number_of_sheets, 1);
  assert_eq!(sheets.len(), 1);
  assert_eq!(sheets[0].name, "A");
}
