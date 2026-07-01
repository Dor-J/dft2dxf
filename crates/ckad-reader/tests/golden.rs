//! Golden and integration tests for cncKad output.

use ckad_reader::parse_content;
use dft2dxf_testkit::professional_cnckad_dft;
use drawing_dxf::write_drawing_to_file;
use drawing_svg::write_drawing_to_string;

#[test]
fn professional_cnckad_dxf_contains_native_primitives() {
  let mut drawing = parse_content(&professional_cnckad_dft(), None).unwrap();
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("pro.dxf");
  write_drawing_to_file(&mut drawing, &path).unwrap();
  let content = std::fs::read_to_string(&path).unwrap();
  assert!(content.contains("ARC") || content.contains("CIRCLE"));
  assert!(content.contains("LAYER"));
  assert!(
    content.contains("$INSUNITS") || content.contains("EXTMIN") || content.contains("$MEASUREMENT")
  );
}

#[test]
fn professional_cnckad_svg_matches_bounds() {
  let drawing = parse_content(&professional_cnckad_dft(), None).unwrap();
  let svg = write_drawing_to_string(&drawing).unwrap();
  assert!(svg.contains("viewBox"));
  assert!(svg.contains("data-layer"));
}

#[test]
fn professional_cnckad_cam_json_structure() {
  let drawing = parse_content(&professional_cnckad_dft(), None).unwrap();
  let cam = drawing.cam.expect("CAM program");
  assert!(cam.tools.len() >= 2);
  assert!(!cam.operations.is_empty());
  let json = serde_json::to_string(&cam).unwrap();
  assert!(json.contains("\"tools\""));
  assert!(json.contains("\"operations\""));
}

#[test]
fn professional_cnckad_entities_have_layers() {
  let drawing = parse_content(&professional_cnckad_dft(), None).unwrap();
  assert!(drawing.sheets[0].entities.iter().any(|e| e.layer.is_some()));
}
