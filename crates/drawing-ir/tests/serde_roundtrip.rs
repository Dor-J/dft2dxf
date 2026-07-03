//! Serde round-trip tests for Drawing IR.

use drawing_ir::{
  ArcSegment, CamOperation, CamProgram, CamTool, Color, Diagnostic, DiagnosticSeverity, Drawing,
  DrawingMetadata, Entity, EntityKind, PaperUnit, Path, PathSegment, Point, Polyline, Sheet,
  SourceProvenance, StrokeStyle, Style, TextRun,
};

#[allow(clippy::too_many_lines)]
fn sample_drawing() -> Drawing {
  let mut drawing = Drawing::new();
  drawing.metadata = DrawingMetadata {
    units: PaperUnit::Millimeters,
    part_name: Some("PART-001".to_string()),
    ..DrawingMetadata::default()
  };
  drawing.cam = Some(CamProgram {
    tools: vec![CamTool {
      kind: "R".to_string(),
      size: 5.0,
      size2: None,
      comment: Some("round".to_string()),
    }],
    operations: vec![CamOperation::Online {
      raw: vec!["ONLINE 0 0 10 10".to_string()],
    }],
  });
  drawing.push_diagnostic(Diagnostic {
    severity: DiagnosticSeverity::Warning,
    code: "test.code".to_string(),
    message: "test".to_string(),
    provenance: Some(SourceProvenance {
      emf_record_index: Some(2),
      emf_record_type: Some(42),
    }),
  });
  drawing.sheets.push(Sheet {
    index: Some(1),
    name: Some("Sheet1".to_string()),
    width: Some(100.0),
    height: Some(50.0),
    entities: vec![
      Entity {
        layer: Some("L1".to_string()),
        style: Style {
          stroke: Some(StrokeStyle {
            color: Color::rgb(255, 0, 0),
            width: 2.0,
          }),
          fill: None,
        },
        kind: EntityKind::Line {
          from: Point::new(0.0, 0.0),
          to: Point::new(10.0, 5.0),
        },
        provenance: Some(SourceProvenance {
          emf_record_index: Some(1),
          emf_record_type: Some(54),
        }),
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Arc(ArcSegment {
          center: Point::new(5.0, 5.0),
          radius: 3.0,
          start_angle: 0.0,
          end_angle: 1.57,
        }),
        provenance: None,
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Text(TextRun {
          position: Point::new(1.0, 2.0),
          text: "Label".to_string(),
          font_family: None,
          font_size: 12.0,
          rotation_deg: 0.0,
          style: Style::default(),
          provenance: None,
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
        kind: EntityKind::Polyline(Polyline {
          points: vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)],
          closed: false,
        }),
        provenance: None,
      },
    ],
    bounds: None,
  });
  drawing
}

#[test]
fn drawing_round_trips_through_json() {
  let original = sample_drawing();
  let json = serde_json::to_string(&original).unwrap();
  let restored: Drawing = serde_json::from_str(&json).unwrap();
  assert_eq!(original, restored);
}

#[test]
fn drawing_round_trips_through_pretty_json() {
  let original = sample_drawing();
  let json = serde_json::to_string_pretty(&original).unwrap();
  let restored: Drawing = serde_json::from_str(&json).unwrap();
  assert_eq!(original, restored);
}
