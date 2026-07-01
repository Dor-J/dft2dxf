//! Unit tests for drawing-ir geometry and bounds.

use drawing_ir::{
  ArcSegment, BoundingBox, DimensionKind, Entity, EntityKind, Path, PathSegment, Point, Polyline,
  Sheet, Style, TextRun,
};

#[test]
fn bounding_box_include_point_and_validity() {
  let mut bounds = BoundingBox::empty();
  assert!(!bounds.is_valid());
  bounds.include_point(Point::new(1.0, 2.0));
  bounds.include_point(Point::new(-1.0, 5.0));
  assert!(bounds.is_valid());
  assert!((bounds.min_x - (-1.0)).abs() < f64::EPSILON);
  assert!((bounds.max_y - 5.0).abs() < f64::EPSILON);
}

#[test]
fn arc_segment_samples_endpoints() {
  let arc = ArcSegment {
    center: Point::new(0.0, 0.0),
    radius: 10.0,
    start_angle: 0.0,
    end_angle: std::f64::consts::FRAC_PI_2,
  };
  let points = arc.sample_points(8);
  assert!(points.len() >= 3);
  let start = arc.point_at_angle(0.0);
  assert!((start.x - 10.0).abs() < 1e-9);
}

#[test]
#[allow(clippy::too_many_lines)]
fn sheet_recompute_bounds_all_entity_kinds() {
  let mut sheet = Sheet {
    index: Some(1),
    name: None,
    width: None,
    height: None,
    entities: vec![
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Line {
          from: Point::new(0.0, 0.0),
          to: Point::new(10.0, 0.0),
        },
        provenance: None,
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Polyline(Polyline {
          points: vec![Point::new(0.0, 0.0), Point::new(5.0, 5.0)],
          closed: false,
        }),
        provenance: None,
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Path(Path {
          segments: vec![
            PathSegment::MoveTo {
              to: Point::new(0.0, 0.0),
            },
            PathSegment::LineTo {
              to: Point::new(1.0, 1.0),
            },
          ],
        }),
        provenance: None,
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Rectangle {
          top_left: Point::new(0.0, 10.0),
          bottom_right: Point::new(10.0, 0.0),
        },
        provenance: None,
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Arc(ArcSegment {
          center: Point::new(5.0, 5.0),
          radius: 2.0,
          start_angle: 0.0,
          end_angle: 1.0,
        }),
        provenance: None,
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Circle {
          center: Point::new(20.0, 20.0),
          radius: 3.0,
        },
        provenance: None,
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Dimension(DimensionKind::Linear {
          from: Point::new(0.0, 0.0),
          to: Point::new(5.0, 0.0),
          offset: 1.0,
          text: None,
        }),
        provenance: None,
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Dimension(DimensionKind::Radial {
          center: Point::new(1.0, 1.0),
          radius: 4.0,
          text: None,
        }),
        provenance: None,
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Text(TextRun {
          position: Point::new(3.0, 3.0),
          text: "A".to_string(),
          font_family: None,
          font_size: 12.0,
          rotation_deg: 0.0,
          style: Style::default(),
          provenance: None,
        }),
        provenance: None,
      },
    ],
    bounds: None,
  };

  sheet.recompute_bounds();
  let bounds = sheet.bounds.expect("bounds computed");
  assert!(bounds.min_x <= 0.0);
  assert!(bounds.max_x >= 20.0);
}

#[test]
fn drawing_metadata_is_empty_and_serde() {
  use drawing_ir::{DrawingMetadata, PaperUnit};

  let empty = DrawingMetadata::default();
  assert!(empty.is_empty());

  let meta = DrawingMetadata {
    part_name: Some("PART".to_string()),
    ..Default::default()
  };
  assert!(!meta.is_empty());

  let json = serde_json::to_string(&meta).unwrap();
  assert!(json.contains("PART"));
  let mut meta = meta;
  meta.units = PaperUnit::Inches;
  assert_eq!(meta.units, PaperUnit::Inches);
}

#[test]
fn cam_program_serde_roundtrip() {
  use drawing_ir::{CamOperation, CamProgram, CamTool};

  let program = CamProgram {
    tools: vec![CamTool {
      kind: "R".to_string(),
      size: 50.0,
      size2: None,
      comment: Some("punch".to_string()),
    }],
    operations: vec![CamOperation::Single {
      position: Some(Point::new(1.0, 2.0)),
      tool_index: Some(0),
      raw: vec!["SINGLE".to_string()],
    }],
  };
  let json = serde_json::to_string(&program).unwrap();
  assert!(json.contains("\"tools\""));
  assert!(json.contains("\"single\""));
}

#[test]
fn drawing_push_diagnostic() {
  use drawing_ir::{Diagnostic, DiagnosticSeverity, Drawing};

  let mut drawing = Drawing::new();
  drawing.push_diagnostic(Diagnostic {
    severity: DiagnosticSeverity::Info,
    code: "test".to_string(),
    message: "note".to_string(),
    provenance: None,
  });
  assert_eq!(drawing.diagnostics.len(), 1);
}

#[test]
fn diagnostic_skips_empty_provenance() {
  use drawing_ir::{Diagnostic, DiagnosticSeverity};

  let diag = Diagnostic {
    severity: DiagnosticSeverity::Warning,
    code: "test.code".to_string(),
    message: "msg".to_string(),
    provenance: None,
  };
  let json = serde_json::to_string(&diag).unwrap();
  assert!(!json.contains("provenance"));
}
