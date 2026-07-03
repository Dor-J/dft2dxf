//! End-to-end EMF replay pipeline tests.

use dft2dxf_testkit::{build_minimal_dft, build_rectangle_emf, MinimalDftSpec};
use emf_reader::{
  replay_to_drawing, EmfDocument, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE,
};

#[test]
fn parses_rectangle_emf_records() {
  let emf = build_rectangle_emf(0, 0, 100, 50);
  let doc = EmfDocument::parse(&emf, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap();
  assert!(doc.records.iter().any(|record| record.record_type == 42));
}

#[test]
fn replays_rectangle_to_drawing_ir() {
  let emf = build_rectangle_emf(0, 0, 100, 50);
  let doc = EmfDocument::parse(&emf, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap();
  let drawing = replay_to_drawing(
    &doc,
    Some(1),
    Some("Sheet1".to_string()),
    Some(297.0),
    Some(210.0),
  );
  assert_eq!(drawing.sheets.len(), 1);
  assert!(!drawing.sheets[0].entities.is_empty());
  let entity = &drawing.sheets[0].entities[0];
  assert!(entity.provenance.is_some());
  assert_eq!(
    entity.provenance.as_ref().unwrap().emf_record_type,
    Some(42)
  );
}

#[test]
fn end_to_end_synthetic_pipeline() {
  let dir = tempfile::tempdir().unwrap();
  let dft_path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(5, 5, 95, 45);
  build_minimal_dft(&dft_path, &MinimalDftSpec::one_sheet("A", emf)).unwrap();

  let mut document = dft_reader::DftDocument::open(&dft_path).unwrap();
  let extracted = document.extract_emf(1).unwrap();
  let emf_doc = EmfDocument::parse(
    &extracted.data,
    DEFAULT_MAX_RECORD_COUNT,
    DEFAULT_MAX_RECORD_SIZE,
  )
  .unwrap();
  let mut drawing = replay_to_drawing(&emf_doc, Some(1), Some("A".to_string()), None, None);
  let svg = drawing_svg::write_drawing_to_string(&drawing).unwrap();
  assert!(svg.contains("svg"));

  let dxf_path = dir.path().join("out.dxf");
  drawing_dxf::write_drawing_to_file(&mut drawing, &dxf_path).unwrap();
  assert!(dxf_path.exists());
}
