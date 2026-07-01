//! Minimal DXF writer for Drawing IR entities.

use std::path::Path;

use drawing_ir::{Diagnostic, EntityKind};
use dxf::entities::{Entity, EntityType, Line, LwPolyline, Text};
use dxf::{Drawing, LwPolylineVertex, Point as DxfPoint};

use crate::error::{DxfError, DxfResult};

/// Writes a drawing IR document to a DXF file.
///
/// Arc entities are omitted and recorded as [`Diagnostic`] entries because mapping a
/// partial arc to a full `CIRCLE` would change geometry.
pub fn write_drawing_to_file(drawing: &mut drawing_ir::Drawing, path: &Path) -> DxfResult<()> {
  let mut dxf = Drawing::new();
  let mut arc_omissions = 0u32;

  for sheet in &drawing.sheets {
    for entity in &sheet.entities {
      if matches!(entity.kind, EntityKind::Arc(_)) {
        arc_omissions += 1;
        continue;
      }
      if let Some(dxf_entity) = map_entity(entity) {
        dxf.add_entity(dxf_entity);
      }
    }
  }

  for _ in 0..arc_omissions {
    drawing.push_diagnostic(Diagnostic::unsupported_dxf_entity(
      "Arc",
      "DXF ARC export is not implemented; entity omitted (no CIRCLE substitution)",
    ));
  }

  dxf
    .save_file(path)
    .map_err(|err| DxfError::Write(err.to_string()))?;
  Ok(())
}

fn map_entity(entity: &drawing_ir::Entity) -> Option<Entity> {
  let specific = match &entity.kind {
    drawing_ir::EntityKind::Line { from, to } => {
      let mut line = Line::default();
      line.p1 = dxf_point(from);
      line.p2 = dxf_point(to);
      EntityType::Line(line)
    }
    drawing_ir::EntityKind::Polyline(polyline) => {
      if polyline.points.len() < 2 {
        return None;
      }
      let mut lw = LwPolyline::default();
      lw.vertices = polyline
        .points
        .iter()
        .map(|point| LwPolylineVertex {
          x: point.x,
          y: point.y,
          ..Default::default()
        })
        .collect();
      lw.set_is_closed(polyline.closed);
      EntityType::LwPolyline(lw)
    }
    drawing_ir::EntityKind::Rectangle {
      top_left,
      bottom_right,
    } => {
      let mut lw = LwPolyline::default();
      lw.vertices = vec![
        LwPolylineVertex {
          x: top_left.x,
          y: top_left.y,
          ..Default::default()
        },
        LwPolylineVertex {
          x: bottom_right.x,
          y: top_left.y,
          ..Default::default()
        },
        LwPolylineVertex {
          x: bottom_right.x,
          y: bottom_right.y,
          ..Default::default()
        },
        LwPolylineVertex {
          x: top_left.x,
          y: bottom_right.y,
          ..Default::default()
        },
      ];
      lw.set_is_closed(true);
      EntityType::LwPolyline(lw)
    }
    drawing_ir::EntityKind::Text(text) => {
      let mut dxf_text = Text::default();
      dxf_text.location = dxf_point(&text.position);
      dxf_text.value = text.text.clone();
      dxf_text.text_height = text.font_size;
      EntityType::Text(dxf_text)
    }
    drawing_ir::EntityKind::Path(path) => {
      let lw = path_to_lwpolyline(path)?;
      EntityType::LwPolyline(lw)
    }
    drawing_ir::EntityKind::Arc(_) => return None,
  };

  Some(Entity::new(specific))
}

fn path_to_lwpolyline(path: &drawing_ir::Path) -> Option<LwPolyline> {
  let mut vertices = Vec::new();
  for segment in &path.segments {
    match segment {
      drawing_ir::PathSegment::MoveTo { to } => {
        vertices.push(LwPolylineVertex {
          x: to.x,
          y: to.y,
          ..Default::default()
        });
      }
      drawing_ir::PathSegment::LineTo { to } => {
        vertices.push(LwPolylineVertex {
          x: to.x,
          y: to.y,
          ..Default::default()
        });
      }
      drawing_ir::PathSegment::Close => {
        if let Some(first) = vertices.first() {
          vertices.push(LwPolylineVertex {
            x: first.x,
            y: first.y,
            ..Default::default()
          });
        }
      }
    }
  }
  if vertices.len() < 2 {
    return None;
  }
  let mut lw = LwPolyline::default();
  lw.vertices = vertices;
  Some(lw)
}

fn dxf_point(point: &drawing_ir::Point) -> DxfPoint {
  DxfPoint::new(point.x, point.y, 0.0)
}
