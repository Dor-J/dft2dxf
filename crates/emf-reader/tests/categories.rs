//! EMF category diagnostic tests.

use dft2dxf_testkit::build_emf_records;
use emf_reader::{replay_to_drawing, EmfDocument, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE};

fn replay_records(records: &[(u32, Vec<u8>)]) -> drawing_ir::Drawing {
  let emf = build_emf_records(records);
  let doc = EmfDocument::parse(&emf, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap();
  replay_to_drawing(&doc, Some(1), None, None, None)
}

#[test]
fn clipping_record_emits_diagnostic() {
  let drawing = replay_records(&[(30, vec![0u8; 16])]);
  assert!(drawing
    .diagnostics
    .iter()
    .any(|d| d.code == "emf.clipping_unsupported"));
}

#[test]
fn brush_record_emits_diagnostic() {
  let drawing = replay_records(&[(39, vec![0u8; 16])]);
  assert!(drawing
    .diagnostics
    .iter()
    .any(|d| d.code == "emf.fill_unsupported"));
}

#[test]
fn raster_record_emits_diagnostic() {
  let drawing = replay_records(&[(76, vec![0u8; 32])]);
  assert!(drawing
    .diagnostics
    .iter()
    .any(|d| d.code == "emf.raster_unsupported"));
}
