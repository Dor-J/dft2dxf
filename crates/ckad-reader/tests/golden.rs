//! Golden DXF output for professional cncKad fixture.

use ckad_reader::parse_content;
use dft2dxf_testkit::professional_cnckad_dft;
use drawing_dxf::write_drawing_to_file;

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
  let svg = drawing_svg::write_drawing_to_string(&drawing).unwrap();
  assert!(svg.contains("viewBox"));
  assert!(svg.contains("data-layer"));
}
