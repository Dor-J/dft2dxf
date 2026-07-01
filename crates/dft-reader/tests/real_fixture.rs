//! Smoke test for a committed real Solid Edge `.dft` fixture.
//!
//! The test is a no-op when `tests/fixtures/valid/real-solid-edge.dft` is absent.
//! See `tests/fixtures/valid/INTAKE.md` for acquisition and provenance requirements.

use std::path::PathBuf;

use dft2dxf_testkit::is_emf;
use dft_reader::{DftDocument, DftOpenOptions, Limits};

/// Canonical path for the first real Solid Edge draft fixture.
pub fn real_solid_edge_dft_fixture_path() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/valid/real-solid-edge.dft")
}

/// Minimum decompressed EMF payload size (one `EMR_HEADER` record).
const MIN_EXTRACTED_EMF_BYTES: usize = 88;

#[test]
fn opens_and_extracts_emf_from_real_solid_edge_dft_fixture() {
  let path = real_solid_edge_dft_fixture_path();
  if !path.exists() {
    eprintln!(
      "SKIP: real Solid Edge DFT fixture not present at {}",
      path.display()
    );
    eprintln!("Add a redistributable file per tests/fixtures/valid/INTAKE.md to enable M1.");
    return;
  }

  let mut document =
    DftDocument::open_with_options(&path, DftOpenOptions::new().with_limits(Limits::strict()))
      .expect("real DFT should open as compound file");

  let report = document.inspect().expect("inspect should succeed");
  assert!(
    report.storage.has_viewer_info,
    "expected JDraftViewerInfo storage in real fixture"
  );
  assert!(
    report.storage.has_document_info,
    "expected JDraftDocumentInfo stream in real fixture"
  );

  let sheets = document.sheets().expect("sheet metadata should parse");
  assert!(!sheets.is_empty(), "expected at least one sheet");

  let extracted = document
    .extract_emf(1)
    .expect("sheet 1 EMF extraction should succeed");

  assert!(
    !extracted.data.is_empty(),
    "extracted EMF must be non-empty"
  );
  assert!(
    extracted.data.len() >= MIN_EXTRACTED_EMF_BYTES,
    "extracted EMF smaller than minimum header size ({MIN_EXTRACTED_EMF_BYTES} bytes)"
  );
  assert!(
    is_emf(&extracted.data),
    "extracted bytes must have EMF signature"
  );

  // `extract_emf` already runs `validate_emf_header`; reaching here implies it passed.
}
