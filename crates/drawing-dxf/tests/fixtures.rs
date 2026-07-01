//! Shared IR fixtures for writer tests.

use drawing_ir::{
  ArcSegment, DimensionKind, Entity, EntityKind, Path, PathSegment, Point, Polyline, Sheet, Style,
  TextRun,
};

/// Drawing with one of each major entity kind.
pub fn multi_entity_drawing() -> drawing_ir::Drawing {
  let mut drawing = drawing_ir::Drawing::new();
  drawing.sheets.push(Sheet {
    index: Some(1),
    name: Some("S".to_string()),
    width: Some(200.0),
    height: Some(100.0),
    entities: vec![
      Entity {
        layer: Some("L1".to_string()),
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
          closed: true,
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
              to: Point::new(3.0, 3.0),
            },
            PathSegment::Close,
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
          center: Point::new(15.0, 15.0),
          radius: 5.0,
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
          radius: 4.0,
        },
        provenance: None,
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Dimension(DimensionKind::Linear {
          from: Point::new(0.0, 0.0),
          to: Point::new(10.0, 0.0),
          offset: 2.0,
          text: Some("10".to_string()),
        }),
        provenance: None,
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Dimension(DimensionKind::Radial {
          center: Point::new(5.0, 5.0),
          radius: 3.0,
          text: None,
        }),
        provenance: None,
      },
      Entity {
        layer: None,
        style: Style::default(),
        kind: EntityKind::Text(TextRun {
          position: Point::new(1.0, 1.0),
          text: "Label".to_string(),
          font_family: Some("Arial".to_string()),
          font_size: 12.0,
          rotation_deg: 45.0,
          style: Style::default(),
          provenance: None,
        }),
        provenance: None,
      },
    ],
    bounds: None,
  });
  drawing
}
