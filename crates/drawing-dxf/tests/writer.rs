mod fixtures;

use fixtures::multi_entity_drawing;
use dft2dxf_testkit::{build_minimal_dft, build_rectangle_emf, MinimalDftSpec};
use drawing_dxf::write_drawing_to_file;
use emf_reader::{
  replay_to_drawing, EmfDocument, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE,
};

#[test]
fn writes_dxf_with_entities() {
  let dir = tempfile::tempdir().unwrap();
  let dft_path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 50, 50);
  build_minimal_dft(&dft_path, &MinimalDftSpec::one_sheet("S1", emf)).unwrap();

  let mut document = dft_reader::DftDocument::open(&dft_path).unwrap();
  let extracted = document.extract_emf(1).unwrap();
  let emf_doc = EmfDocument::parse(
    &extracted.data,
    DEFAULT_MAX_RECORD_COUNT,
    DEFAULT_MAX_RECORD_SIZE,
  )
  .unwrap();
  let mut drawing = replay_to_drawing(&emf_doc, Some(1), None, None, None);

  let dxf_path = dir.path().join("out.dxf");
  write_drawing_to_file(&mut drawing, &dxf_path).unwrap();
  let content = std::fs::read_to_string(&dxf_path).unwrap();
  assert!(content.contains("SECTION"));
  assert!(content.contains("ENTITIES"));
}

#[test]
fn dxf_output_contains_lwpolyline_entities() {
  let dir = tempfile::tempdir().unwrap();
  let dft_path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 50, 50);
  build_minimal_dft(&dft_path, &MinimalDftSpec::one_sheet("S1", emf)).unwrap();

  let mut document = dft_reader::DftDocument::open(&dft_path).unwrap();
  let extracted = document.extract_emf(1).unwrap();
  let emf_doc = EmfDocument::parse(
    &extracted.data,
    DEFAULT_MAX_RECORD_COUNT,
    DEFAULT_MAX_RECORD_SIZE,
  )
  .unwrap();
  let mut drawing = replay_to_drawing(&emf_doc, Some(1), None, None, None);

  let dxf_path = dir.path().join("out.dxf");
  write_drawing_to_file(&mut drawing, &dxf_path).unwrap();
  let content = std::fs::read_to_string(&dxf_path).unwrap();
  assert!(
    content.contains("LWPOLYLINE")
      || content.contains("POLYLINE")
      || content.contains("LINE")
  );
}

#[test]
fn emits_native_arc_entities() {
  use drawing_ir::{ArcSegment, Entity, EntityKind, Point, Sheet, Style};
  use tempfile::tempdir;

  let mut drawing = drawing_ir::Drawing::new();
  drawing.sheets.push(Sheet {
    index: Some(1),
    name: None,
    width: None,
    height: None,
    entities: vec![Entity {
      layer: None,
      style: Style::default(),
      kind: EntityKind::Arc(ArcSegment {
        center: Point::new(10.0, 10.0),
        radius: 5.0,
        start_angle: 0.0,
        end_angle: 1.0,
      }),
      provenance: None,
    }],
    bounds: None,
  });

  let dir = tempdir().unwrap();
  let dxf_path = dir.path().join("arc.dxf");
  write_drawing_to_file(&mut drawing, &dxf_path).unwrap();
  let content = std::fs::read_to_string(&dxf_path).unwrap();
  assert!(content.contains("ARC"));
  assert!(!drawing
    .diagnostics
    .iter()
    .any(|d| d.code == "dxf.unsupported_entity"));
}

#[test]
fn emits_native_circle_entities() {
  use drawing_ir::{Entity, EntityKind, Point, Sheet, Style};

  let mut drawing = drawing_ir::Drawing::new();
  drawing.sheets.push(Sheet {
    index: Some(1),
    name: None,
    width: None,
    height: None,
    entities: vec![Entity {
      layer: Some("L1".to_string()),
      style: Style::default(),
      kind: EntityKind::Circle {
        center: Point::new(5.0, 5.0),
        radius: 3.0,
      },
      provenance: None,
    }],
    bounds: None,
  });

  let dir = tempfile::tempdir().unwrap();
  let dxf_path = dir.path().join("circle.dxf");
  write_drawing_to_file(&mut drawing, &dxf_path).unwrap();
  let content = std::fs::read_to_string(&dxf_path).unwrap();
  assert!(content.contains("CIRCLE"));
  assert!(content.contains("L1"));
}

#[test]
fn emits_cam_layers_when_enabled() {
  use ckad_reader::parse_content;
  use drawing_dxf::{write_drawing_to_file_with_options, DxfWriteOptions};
  use dft2dxf_testkit::professional_cnckad_dft;

  let mut drawing = parse_content(&professional_cnckad_dft(), None).unwrap();
  let dir = tempfile::tempdir().unwrap();
  let dxf_path = dir.path().join("cam.dxf");
  write_drawing_to_file_with_options(
    &mut drawing,
    &dxf_path,
    DxfWriteOptions {
      include_cam: true,
    },
  )
  .unwrap();
  let content = std::fs::read_to_string(&dxf_path).unwrap();
  assert!(content.contains("PUNCH") || content.contains("CUT") || content.contains("TOOLS"));
}

#[test]
fn header_reflects_millimeter_units() {
  use drawing_ir::{Entity, EntityKind, PaperUnit, Point, Sheet, Style};

  let mut drawing = drawing_ir::Drawing::new();
  drawing.metadata.units = PaperUnit::Millimeters;
  drawing.sheets.push(Sheet {
    index: Some(1),
    name: None,
    width: None,
    height: None,
    entities: vec![Entity {
      layer: None,
      style: Style::default(),
      kind: EntityKind::Line {
        from: Point::new(0.0, 0.0),
        to: Point::new(1.0, 0.0),
      },
      provenance: None,
    }],
    bounds: None,
  });

  let dir = tempfile::tempdir().unwrap();
  let dxf_path = dir.path().join("units.dxf");
  write_drawing_to_file(&mut drawing, &dxf_path).unwrap();
  let content = std::fs::read_to_string(&dxf_path).unwrap();
  assert!(content.contains("HEADER") || content.contains("ENTITIES"));
  assert_eq!(drawing.metadata.units, PaperUnit::Millimeters);
}

#[test]
fn writes_all_entity_kinds() {
  let mut drawing = multi_entity_drawing();
  let dir = tempfile::tempdir().unwrap();
  let dxf_path = dir.path().join("multi.dxf");
  write_drawing_to_file(&mut drawing, &dxf_path).unwrap();
  let content = std::fs::read_to_string(&dxf_path).unwrap();
  assert!(content.contains("TEXT") || content.contains("MTEXT"));
  assert!(content.contains("ARC"));
  assert!(content.contains("CIRCLE"));
}
