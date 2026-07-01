//! Minimal DXF writer for Drawing IR entities.

use std::path::Path;

use dxf::entities::*;
use dxf::{Drawing, Entity, EntityType, Point as DxfPoint};

use crate::error::{DxfError, DxfResult};

/// Writes a drawing IR document to a DXF file.
pub fn write_drawing_to_file(drawing: &drawing_ir::Drawing, path: &Path) -> DxfResult<()> {
  let mut dxf = Drawing::new();

  for sheet in &drawing.sheets {
    for entity in &sheet.entities {
      if let Some(dxf_entity) = map_entity(entity) {
        dxf.add_entity(dxf_entity);
      }
    }
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
      lw.is_closed = polyline.closed;
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
      lw.is_closed = true;
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
    drawing_ir::EntityKind::Arc(arc) => {
      let mut circle = Circle::default();
      circle.center = dxf_point(&arc.center);
      circle.radius = arc.radius;
      EntityType::Circle(circle)
    }
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
