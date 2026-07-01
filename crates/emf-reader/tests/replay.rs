//! EMF replay tests for extended record types.

use dft2dxf_testkit::{
  build_arc_emf, build_pen_and_line_emf, build_polyline_emf, build_text_emf, build_transform_emf,
};
use drawing_ir::EntityKind;
use emf_reader::{
  replay_to_drawing, EmfDocument, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE,
};

fn parse_and_replay(emf: &[u8]) -> drawing_ir::Drawing {
  let doc = EmfDocument::parse(emf, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap();
  replay_to_drawing(&doc, Some(1), None, None, None)
}

#[test]
fn replays_polyline_emf() {
  let emf = build_polyline_emf(&[(0, 0), (50, 0), (50, 50)]);
  let drawing = parse_and_replay(&emf);
  assert!(drawing.sheets[0]
    .entities
    .iter()
    .any(|e| matches!(e.kind, EntityKind::Polyline(_))));
}

#[test]
fn replays_arc_emf() {
  let emf = build_arc_emf(0, 0, 100, 100, 100, 50, 50, 100);
  let drawing = parse_and_replay(&emf);
  assert!(drawing.sheets[0]
    .entities
    .iter()
    .any(|e| matches!(e.kind, EntityKind::Arc(_))));
}

#[test]
fn replays_pen_and_line_emf() {
  let emf = build_pen_and_line_emf();
  let drawing = parse_and_replay(&emf);
  assert!(drawing.sheets[0]
    .entities
    .iter()
    .any(|e| matches!(e.kind, EntityKind::Line { .. })));
}

#[test]
fn replays_text_emf() {
  let emf = build_text_emf(10, 20, "Hello");
  let drawing = parse_and_replay(&emf);
  assert!(drawing.sheets[0]
    .entities
    .iter()
    .any(|e| matches!(e.kind, EntityKind::Text(_))));
}

#[test]
fn replays_transform_emf_with_rectangle() {
  let emf = build_transform_emf();
  let drawing = parse_and_replay(&emf);
  assert!(!drawing.sheets[0].entities.is_empty());
}

#[test]
fn replays_polygon_and_ellipse_emf() {
  use dft2dxf_testkit::{build_ellipse_emf, build_polygon_emf};
  let poly = parse_and_replay(&build_polygon_emf(&[(0, 0), (50, 0), (25, 50)]));
  assert!(poly.sheets[0]
    .entities
    .iter()
    .any(|e| matches!(e.kind, EntityKind::Polyline(_))));
  let ellipse = parse_and_replay(&build_ellipse_emf(0, 0, 100, 50));
  assert!(ellipse.sheets[0]
    .entities
    .iter()
    .any(|e| matches!(e.kind, EntityKind::Circle { .. })));
}

#[test]
fn rejects_truncated_emf() {
  let data = build_transform_emf();
  let truncated = &data[..data.len() / 2];
  let err = EmfDocument::parse(truncated, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE);
  assert!(err.is_err());
}
