//! Extended EMF replay coverage for less common record types.

use dft2dxf_testkit::{
  build_arc_emf, build_emf_records, build_pen_and_line_emf, build_rectangle_emf, build_text_emf,
};
use drawing_ir::{EntityKind, PathSegment};
use emf_reader::{
  replay_to_drawing, EmfDocument, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE,
};

fn parse_and_replay(emf: &[u8]) -> drawing_ir::Drawing {
  let doc = EmfDocument::parse(emf, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap();
  replay_to_drawing(&doc, Some(1), None, None, None)
}

#[allow(clippy::too_many_arguments)]
fn arc_payload(
  left: i32,
  top: i32,
  right: i32,
  bottom: i32,
  start_x: i32,
  start_y: i32,
  end_x: i32,
  end_y: i32,
) -> Vec<u8> {
  let mut payload = vec![0u8; 32];
  payload[0..4].copy_from_slice(&left.to_le_bytes());
  payload[4..8].copy_from_slice(&top.to_le_bytes());
  payload[8..12].copy_from_slice(&right.to_le_bytes());
  payload[12..16].copy_from_slice(&bottom.to_le_bytes());
  payload[16..20].copy_from_slice(&start_x.to_le_bytes());
  payload[20..24].copy_from_slice(&start_y.to_le_bytes());
  payload[24..28].copy_from_slice(&end_x.to_le_bytes());
  payload[28..32].copy_from_slice(&end_y.to_le_bytes());
  payload
}

fn world_transform_payload(m11: f32, m22: f32) -> Vec<u8> {
  let mut payload = vec![0u8; 32];
  payload[0..4].copy_from_slice(&m11.to_le_bytes());
  payload[12..16].copy_from_slice(&m22.to_le_bytes());
  payload
}

fn ext_create_pen_payload(index: u32, width: u32, color: u32) -> Vec<u8> {
  let mut payload = vec![0u8; 28];
  payload[0..4].copy_from_slice(&index.to_le_bytes());
  payload[8..12].copy_from_slice(&width.to_le_bytes());
  payload[16..20].copy_from_slice(&color.to_le_bytes());
  payload
}

fn ext_create_font_payload(index: u32, height: i32) -> Vec<u8> {
  let mut payload = vec![0u8; 12];
  payload[0..4].copy_from_slice(&index.to_le_bytes());
  payload[4..8].copy_from_slice(&height.to_le_bytes());
  payload
}

fn select_object_payload(index: u32) -> Vec<u8> {
  let mut payload = vec![0u8; 4];
  payload[0..4].copy_from_slice(&index.to_le_bytes());
  payload
}

fn polyline16_payload(points: &[(i16, i16)]) -> Vec<u8> {
  let mut payload = vec![0u8; 4 + points.len() * 4];
  payload[0..4].copy_from_slice(&u32::try_from(points.len()).unwrap().to_le_bytes());
  let mut offset = 4usize;
  for (x, y) in points {
    payload[offset..offset + 2].copy_from_slice(&x.to_le_bytes());
    payload[offset + 2..offset + 4].copy_from_slice(&y.to_le_bytes());
    offset += 4;
  }
  payload
}

fn build_unicode_text_emf(x: i32, y: i32, text: &str) -> Vec<u8> {
  let utf16: Vec<u16> = text.encode_utf16().collect();
  let byte_len = utf16.len() * 2;
  let string_offset = 56u32;
  let payload_len = usize::try_from(string_offset).unwrap_or(56) + byte_len + 2;
  let padded_len = payload_len.next_multiple_of(4);
  let mut payload = vec![0u8; padded_len];
  payload[28..32].copy_from_slice(&x.to_le_bytes());
  payload[32..36].copy_from_slice(&y.to_le_bytes());
  payload[36..40].copy_from_slice(&u32::try_from(utf16.len()).unwrap().to_le_bytes());
  payload[40..44].copy_from_slice(&string_offset.to_le_bytes());
  let string_start = usize::try_from(string_offset).unwrap_or(56) - 8;
  for (index, unit) in utf16.iter().enumerate() {
    let offset = string_start + index * 2;
    payload[offset..offset + 2].copy_from_slice(&unit.to_le_bytes());
  }
  build_emf_records(&[(84, payload)])
}

#[test]
fn header_record_count_mismatch_emits_diagnostic() {
  let mut emf = build_rectangle_emf(0, 0, 10, 10);
  emf[52..56].copy_from_slice(&999u32.to_le_bytes());
  let drawing = parse_and_replay(&emf);
  assert!(drawing
    .diagnostics
    .iter()
    .any(|d| d.code == "emf.header_record_count"));
}

#[test]
fn replays_set_world_transform_scales_geometry() {
  let emf = build_emf_records(&[
    (35, world_transform_payload(2.0, 2.0)),
    (
      42,
      {
        let mut rect = vec![0u8; 16];
        rect[0..4].copy_from_slice(&0i32.to_le_bytes());
        rect[4..8].copy_from_slice(&0i32.to_le_bytes());
        rect[8..12].copy_from_slice(&10i32.to_le_bytes());
        rect[12..16].copy_from_slice(&10i32.to_le_bytes());
        rect
      },
    ),
  ]);
  let drawing = parse_and_replay(&emf);
  let rect = drawing.sheets[0]
    .entities
    .iter()
    .find(|entity| matches!(entity.kind, EntityKind::Rectangle { .. }))
    .expect("rectangle");
  if let EntityKind::Rectangle {
    top_left,
    bottom_right,
  } = &rect.kind
  {
    assert!((bottom_right.x - 20.0).abs() < f64::EPSILON);
    assert!((bottom_right.y - 20.0).abs() < f64::EPSILON);
    assert!((top_left.x).abs() < f64::EPSILON);
    assert!((top_left.y).abs() < f64::EPSILON);
  }
}

#[test]
fn replays_modify_world_transform() {
  let emf = build_emf_records(&[(36, world_transform_payload(1.5, 1.5))]);
  let drawing = parse_and_replay(&emf);
  assert!(drawing.sheets[0].entities.is_empty());
}

#[test]
fn replays_ext_create_pen_stroke() {
  let emf = build_emf_records(&[
    (95, ext_create_pen_payload(1, 3, 0x0000_FF00)),
    (37, select_object_payload(1)),
    (
      54,
      {
        let mut line = vec![0u8; 8];
        line[0..4].copy_from_slice(&5i32.to_le_bytes());
        line[4..8].copy_from_slice(&5i32.to_le_bytes());
        line
      },
    ),
  ]);
  let drawing = parse_and_replay(&emf);
  let line = drawing.sheets[0]
    .entities
    .iter()
    .find(|entity| matches!(entity.kind, EntityKind::Line { .. }))
    .expect("line");
  let stroke = line.style.stroke.as_ref().expect("stroke");
  assert_eq!(stroke.color.g, 255);
  assert!((stroke.width - 3.0).abs() < f64::EPSILON);
}

#[test]
fn font_selection_sets_text_size() {
  let text_payload = {
    let text = build_text_emf(5, 5, "Sized");
    let doc =
      EmfDocument::parse(&text, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap();
    doc.records[1].data[8..].to_vec()
  };
  let drawing = parse_and_replay(&build_emf_records(&[
    (82, ext_create_font_payload(2, 24)),
    (37, select_object_payload(2)),
    (83, text_payload),
  ]));
  let text = drawing.sheets[0]
    .entities
    .iter()
    .find(|entity| matches!(entity.kind, EntityKind::Text(_)))
    .expect("text");
  if let EntityKind::Text(run) = &text.kind {
    assert!((run.font_size - 24.0).abs() < f64::EPSILON);
    assert_eq!(run.text, "Sized");
  }
}

#[test]
fn replays_polyline16_and_polygon16() {
  let polyline = parse_and_replay(&build_emf_records(&[(
    87,
    polyline16_payload(&[(0, 0), (50, 0), (50, 50)]),
  )]));
  assert!(polyline.sheets[0]
    .entities
    .iter()
    .any(|entity| matches!(entity.kind, EntityKind::Polyline(_))));

  let polygon = parse_and_replay(&build_emf_records(&[(
    86,
    polyline16_payload(&[(0, 0), (40, 0), (20, 30)]),
  )]));
  let poly = polygon.sheets[0]
    .entities
    .iter()
    .find(|entity| matches!(entity.kind, EntityKind::Polyline(_)))
    .expect("polygon");
  if let EntityKind::Polyline(polyline) = &poly.kind {
    assert!(polyline.closed);
  }
}

#[test]
fn replays_polybezier16_path() {
  let emf = build_emf_records(&[(
    88,
    polyline16_payload(&[(0, 0), (10, 10), (20, 0), (30, 10)]),
  )]);
  let drawing = parse_and_replay(&emf);
  assert!(drawing.sheets[0]
    .entities
    .iter()
    .any(|entity| matches!(entity.kind, EntityKind::Path(_))));
}

#[test]
fn replays_arcto_chord_and_pie() {
  let bounds = arc_payload(0, 0, 100, 100, 100, 50, 50, 100);
  for record_type in [55u32, 46, 47] {
    let drawing = parse_and_replay(&build_emf_records(&[(record_type, bounds.clone())]));
    assert!(
      drawing.sheets[0]
        .entities
        .iter()
        .any(|entity| matches!(entity.kind, EntityKind::Arc(_))),
      "record type {record_type}"
    );
  }
}

#[test]
fn replays_exttextoutw_unicode() {
  let drawing = parse_and_replay(&build_unicode_text_emf(12, 18, "Ünicode"));
  let text = drawing.sheets[0]
    .entities
    .iter()
    .find(|entity| matches!(entity.kind, EntityKind::Text(_)))
    .expect("text");
  if let EntityKind::Text(run) = &text.kind {
    assert_eq!(run.text, "Ünicode");
  }
}

#[test]
fn exclude_clip_emits_diagnostic() {
  let drawing = parse_and_replay(&build_emf_records(&[(31, vec![0u8; 16])]));
  assert!(drawing
    .diagnostics
    .iter()
    .any(|d| d.code == "emf.clipping_unsupported"));
}

#[test]
fn stretchdibits_emits_raster_diagnostic() {
  let drawing = parse_and_replay(&build_emf_records(&[(81, vec![0u8; 32])]));
  assert!(drawing
    .diagnostics
    .iter()
    .any(|d| d.code == "emf.raster_unsupported"));
}

#[test]
fn unsupported_record_class_emits_diagnostic() {
  let drawing = parse_and_replay(&build_emf_records(&[(9_999, vec![0u8; 8])]));
  assert!(drawing
    .diagnostics
    .iter()
    .any(|d| d.code == "emf.unsupported_record"));
}

#[test]
fn synthesizes_path_from_connected_lines() {
  let emf = build_emf_records(&[
    (
      27,
      {
        let mut moveto = vec![0u8; 8];
        moveto[0..4].copy_from_slice(&0i32.to_le_bytes());
        moveto[4..8].copy_from_slice(&0i32.to_le_bytes());
        moveto
      },
    ),
    (
      54,
      {
        let mut line = vec![0u8; 8];
        line[0..4].copy_from_slice(&10i32.to_le_bytes());
        line[4..8].copy_from_slice(&0i32.to_le_bytes());
        line
      },
    ),
    (
      54,
      {
        let mut line = vec![0u8; 8];
        line[0..4].copy_from_slice(&10i32.to_le_bytes());
        line[4..8].copy_from_slice(&10i32.to_le_bytes());
        line
      },
    ),
  ]);
  let drawing = parse_and_replay(&emf);
  let path = drawing.sheets[0]
    .entities
    .iter()
    .find(|entity| matches!(entity.kind, EntityKind::Path(_)))
    .expect("synthesized path");
  if let EntityKind::Path(path) = &path.kind {
    assert!(path.segments.len() >= 3);
    assert!(matches!(path.segments[0], PathSegment::MoveTo { .. }));
    assert!(path
      .segments
      .iter()
      .any(|segment| matches!(segment, PathSegment::LineTo { .. })));
  }
}

#[test]
fn moveto_lineto_chain_carries_path_provenance() {
  let drawing = parse_and_replay(&build_pen_and_line_emf());
  let path = drawing.sheets[0]
    .entities
    .iter()
    .find(|entity| matches!(entity.kind, EntityKind::Path(_)));
  if let Some(path) = path {
    assert!(path.provenance.is_some());
  }
}

#[test]
fn arc_emf_replay_matches_native_builder() {
  let emf = build_arc_emf(0, 0, 100, 100, 100, 50, 50, 100);
  let drawing = parse_and_replay(&emf);
  assert!(drawing.sheets[0]
    .entities
    .iter()
    .any(|entity| matches!(entity.kind, EntityKind::Arc(_))));
}
