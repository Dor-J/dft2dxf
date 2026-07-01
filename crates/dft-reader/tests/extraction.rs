use dft2dxf_testkit::{build_minimal_dft, build_rectangle_emf, is_emf, MinimalDftSpec};
use dft_reader::{DftDocument, DftError, DftOpenOptions, Limits};

#[test]
fn opens_synthetic_dft_and_extracts_emf() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(10, 20, 110, 70);
  build_minimal_dft(&path, &MinimalDftSpec::one_sheet("Sheet1", emf)).unwrap();

  let mut document =
    DftDocument::open_with_options(&path, DftOpenOptions::new().with_limits(Limits::strict()))
      .unwrap();
  let report = document.inspect().unwrap();
  assert!(report.storage.has_viewer_info);
  assert!(report.storage.has_document_info);
  assert_eq!(report.sheets.len(), 1);

  let extracted = document.extract_emf(1).unwrap();
  assert!(is_emf(&extracted.data));
}

#[test]
fn rejects_non_compound_file() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("not-a-dft.bin");
  std::fs::write(&path, b"not a compound file").unwrap();
  let result = DftDocument::open(&path);
  assert!(matches!(result, Err(DftError::NotCompoundFile { .. })));
}

#[test]
fn sheet_out_of_range_is_reported() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 10, 10);
  build_minimal_dft(&path, &MinimalDftSpec::one_sheet("Sheet1", emf)).unwrap();

  let mut document = DftDocument::open(&path).unwrap();
  let err = document.extract_emf(99).unwrap_err();
  assert!(matches!(err, DftError::SheetOutOfRange { .. }));
}
