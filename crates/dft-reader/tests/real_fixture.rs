//! Smoke tests for real Solid Edge `.dft` fixtures.
//!
//! Default fixture directory: `tests/fixtures/valid/` (redistributable samples only).
//!
//! Local proprietary fixtures: `tests/fixtures/valid/local/` (gitignored). Enable with:
//!
//! ```bash
//! cargo test -p dft-reader opens_and_extracts_emf_from_real_solid_edge_dft_fixture -- --local
//! # or
//! DFT2DXF_LOCAL=1 cargo test -p dft-reader opens_and_extracts_emf_from_real_solid_edge_dft_fixture
//! ```
//!
//! See `tests/fixtures/valid/INTAKE.md`.

use std::path::Path;

use ckad_reader::detect_format;
use dft2dxf_testkit::{discover_valid_dft_fixtures, is_emf, use_local_fixtures};
use dft_reader::{DftDocument, DftOpenOptions, Limits};

/// Minimum decompressed EMF payload size (one `EMR_HEADER` record).
const MIN_EXTRACTED_EMF_BYTES: usize = 88;

#[test]
fn opens_and_extracts_emf_from_real_solid_edge_dft_fixture() {
  let use_local = use_local_fixtures();
  let fixtures = discover_valid_dft_fixtures(use_local);
  if fixtures.is_empty() {
    let mode = if use_local { "local" } else { "valid" };
    eprintln!(
      "SKIP: no .dft files in tests/fixtures/valid/{suffix}",
      suffix = if use_local { "local/" } else { "" }
    );
    if use_local {
      eprintln!("Add customer .dft files under tests/fixtures/valid/local/ (gitignored).");
    } else {
      eprintln!(
        "Add a redistributable .dft under tests/fixtures/valid/ or run with --local / DFT2DXF_LOCAL=1."
      );
      eprintln!("See tests/fixtures/valid/INTAKE.md");
    }
    eprintln!("Fixture mode requested: {mode}");
    return;
  }

  for path in fixtures {
    if is_cnckad_text_dft(&path) {
      eprintln!("SKIP (cncKad text): {}", path.display());
      continue;
    }
    assert_extracts_emf_from_dft(&path);
  }
}

fn is_cnckad_text_dft(path: &Path) -> bool {
  let header = std::fs::read(path).unwrap_or_default();
  detect_format(&header[..header.len().min(12)]).is_some()
}

fn assert_extracts_emf_from_dft(path: &Path) {
  let mut document =
    DftDocument::open_with_options(path, &DftOpenOptions::new().with_limits(Limits::strict()))
      .unwrap_or_else(|err| panic!("{} should open as compound file: {err}", path.display()));

  let report = document
    .inspect()
    .unwrap_or_else(|err| panic!("{} inspect failed: {err}", path.display()));
  assert!(
    report.storage.has_viewer_info,
    "{}: expected JDraftViewerInfo storage",
    path.display()
  );
  assert!(
    report.storage.has_document_info,
    "{}: expected JDraftDocumentInfo stream",
    path.display()
  );

  let sheets = document
    .sheets()
    .unwrap_or_else(|err| panic!("{} sheet metadata failed: {err}", path.display()));
  assert!(
    !sheets.is_empty(),
    "{}: expected at least one sheet",
    path.display()
  );

  let extracted = document
    .extract_emf(1)
    .unwrap_or_else(|err| panic!("{} sheet 1 EMF extraction failed: {err}", path.display()));

  assert!(
    !extracted.data.is_empty(),
    "{}: extracted EMF must be non-empty",
    path.display()
  );
  assert!(
    extracted.data.len() >= MIN_EXTRACTED_EMF_BYTES,
    "{}: extracted EMF smaller than minimum header size ({MIN_EXTRACTED_EMF_BYTES} bytes)",
    path.display()
  );
  assert!(
    is_emf(&extracted.data),
    "{}: extracted bytes must have EMF signature",
    path.display()
  );
}
