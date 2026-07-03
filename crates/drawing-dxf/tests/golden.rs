//! Golden DXF regression tests.

use std::path::PathBuf;

use dft2dxf_testkit::{build_minimal_dft, build_rectangle_emf, MinimalDftSpec};
use drawing_dxf::write_drawing_to_bytes;
use emf_reader::{
  replay_to_drawing, EmfDocument, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE,
};

fn golden_dxf_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/dxf")
}

fn normalize_dxf(content: &str) -> String {
  content
    .lines()
    .filter(|line| {
      !line.contains("$HANDSEED")
        && !line.contains("$FINGERPRINTGUID")
        && !line.contains("$VERSIONGUID")
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn rectangle_drawing() -> drawing_ir::Drawing {
  let emf = build_rectangle_emf(0, 0, 100, 50);
  let doc = EmfDocument::parse(&emf, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap();
  replay_to_drawing(&doc, Some(1), None, None, None)
}

#[test]
fn golden_rectangle_dxf_contains_entities() {
  let mut drawing = rectangle_drawing();
  let bytes = write_drawing_to_bytes(&mut drawing, drawing_dxf::DxfWriteOptions::default()).unwrap();
  let content = String::from_utf8_lossy(&bytes);
  let normalized = normalize_dxf(&content);
  assert!(normalized.contains("ENTITIES"));
  assert!(normalized.contains("LWPOLYLINE") || normalized.contains("LINE"));
}

#[test]
fn pipeline_solid_edge_dxf_matches_structure() {
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
  let mut drawing = replay_to_drawing(&emf_doc, Some(1), None, None, None);
  let bytes = write_drawing_to_bytes(&mut drawing, drawing_dxf::DxfWriteOptions::default()).unwrap();
  let golden_path = golden_dxf_dir().join("rectangle.dxf");
  if golden_path.exists() {
    let golden = std::fs::read_to_string(golden_path).unwrap();
    assert_eq!(
      normalize_dxf(&String::from_utf8_lossy(&bytes)),
      normalize_dxf(&golden)
    );
  }
}
