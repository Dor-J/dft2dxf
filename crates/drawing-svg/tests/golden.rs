use std::path::PathBuf;

use drawing_ir::{EntityKind, PathSegment, Point};
use drawing_svg::write_drawing_to_string;

fn golden_svg_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/svg")
}

fn line_drawing() -> drawing_ir::Drawing {
  let mut drawing = drawing_ir::Drawing::new();
  drawing.sheets.push(drawing_ir::Sheet {
    index: Some(1),
    name: Some("Sheet1".to_string()),
    width: Some(100.0),
    height: Some(50.0),
    entities: vec![drawing_ir::Entity {
      layer: None,
      style: drawing_ir::Style::default(),
      kind: EntityKind::Line {
        from: Point::new(0.0, 0.0),
        to: Point::new(100.0, 50.0),
      },
      provenance: None,
    }],
    bounds: None,
  });
  drawing
}

#[test]
fn svg_output_is_deterministic_for_line() {
  let drawing = line_drawing();
  let first = write_drawing_to_string(&drawing).unwrap();
  let second = write_drawing_to_string(&drawing).unwrap();
  assert_eq!(first, second);
  assert!(first.contains("line"));
}

#[test]
fn svg_line_matches_golden_file() {
  let svg = write_drawing_to_string(&line_drawing()).unwrap();
  let golden =
    std::fs::read_to_string(golden_svg_dir().join("line.svg")).expect("missing line.svg golden");
  assert_eq!(svg, golden);
}

#[test]
fn svg_line_matches_golden_snapshot() {
  let svg = write_drawing_to_string(&line_drawing()).unwrap();
  insta::with_settings!({snapshot_path => golden_svg_dir()}, {
    insta::assert_snapshot!("line", svg);
  });
}

#[test]
fn svg_renders_path_segments() {
  let mut drawing = drawing_ir::Drawing::new();
  drawing.sheets.push(drawing_ir::Sheet {
    index: Some(1),
    name: None,
    width: None,
    height: None,
    entities: vec![drawing_ir::Entity {
      layer: None,
      style: drawing_ir::Style::default(),
      kind: EntityKind::Path(drawing_ir::Path {
        segments: vec![
          PathSegment::MoveTo {
            to: Point::new(0.0, 0.0),
          },
          PathSegment::LineTo {
            to: Point::new(10.0, 0.0),
          },
        ],
      }),
      provenance: None,
    }],
    bounds: None,
  });

  let svg = write_drawing_to_string(&drawing).unwrap();
  insta::with_settings!({snapshot_path => golden_svg_dir()}, {
    insta::assert_snapshot!("path", svg);
  });
}
