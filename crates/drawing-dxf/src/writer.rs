//! Professional DXF writer for Drawing IR entities.

use std::collections::BTreeSet;
use std::path::Path;

use drawing_ir::{
  ArcSegment, CamOperation, DimensionKind, Drawing, Entity, EntityKind, PaperUnit, Point,
};
use dxf::entities::{Arc, Circle, Entity as DxfEntity, EntityType, Line, LwPolyline, Text};
use dxf::enums::{DrawingUnits, Units};
use dxf::tables::Layer;
use dxf::{Color, Drawing as DxfDrawing, LwPolylineVertex, Point as DxfPoint};

use crate::error::{DxfError, DxfResult};

/// Options controlling DXF export.
#[derive(Debug, Clone, Copy)]
pub struct DxfWriteOptions {
  /// Whether to emit CAM operations on dedicated layers.
  pub include_cam: bool,
}

impl Default for DxfWriteOptions {
  fn default() -> Self {
    Self { include_cam: true }
  }
}

/// Writes a drawing IR document to a DXF file.
pub fn write_drawing_to_file(drawing: &mut Drawing, path: &Path) -> DxfResult<()> {
  write_drawing_to_file_with_options(drawing, path, DxfWriteOptions::default())
}

/// Writes a drawing IR document to a DXF file with explicit options.
pub fn write_drawing_to_file_with_options(
  drawing: &mut Drawing,
  path: &Path,
  options: DxfWriteOptions,
) -> DxfResult<()> {
  let mut dxf = DxfDrawing::new();
  configure_header(&mut dxf, drawing);

  let layer_names = collect_layer_names(drawing, options.include_cam);
  ensure_layers(&mut dxf, &layer_names);

  let bounds = compute_drawing_bounds(drawing);
  if let Some(bounds) = bounds {
    dxf.header.minimum_drawing_extents = dxf_point(&Point::new(bounds.min_x, bounds.min_y));
    dxf.header.maximum_drawing_extents = dxf_point(&Point::new(bounds.max_x, bounds.max_y));
  }

  for sheet in &drawing.sheets {
    for entity in &sheet.entities {
      if let Some(mut dxf_entity) = map_entity(entity) {
        apply_entity_style(&mut dxf_entity, entity);
        dxf.add_entity(dxf_entity);
      }
    }
  }

  if options.include_cam {
    if let Some(cam) = &drawing.cam {
      append_cam_entities(&mut dxf, cam);
    }
  }

  dxf.normalize();

  dxf
    .save_file(path)
    .map_err(|err| DxfError::Write(err.to_string()))?;
  Ok(())
}

fn configure_header(dxf: &mut DxfDrawing, drawing: &Drawing) {
  dxf.header.default_drawing_units = match drawing.metadata.units {
    PaperUnit::Millimeters => Units::Millimeters,
    PaperUnit::Inches => Units::Inches,
    PaperUnit::Unitless => Units::Unitless,
  };
  dxf.header.drawing_units = DrawingUnits::Metric;
}

fn collect_layer_names(drawing: &Drawing, include_cam: bool) -> BTreeSet<String> {
  let mut names = BTreeSet::new();
  names.insert("0".to_string());
  for sheet in &drawing.sheets {
    for entity in &sheet.entities {
      if let Some(layer) = &entity.layer {
        names.insert(layer.clone());
      }
    }
  }
  if include_cam && drawing.cam.is_some() {
    names.insert("PUNCH".to_string());
    names.insert("CUT".to_string());
    names.insert("TOOLS".to_string());
  }
  names
}

fn ensure_layers(dxf: &mut DxfDrawing, layer_names: &BTreeSet<String>) {
  for name in layer_names {
    if dxf.layers().any(|layer| layer.name == *name) {
      continue;
    }
    let mut layer = Layer::default();
    layer.name = name.clone();
    layer.color = Color::from_index(7);
    dxf.add_layer(layer);
  }
}

fn compute_drawing_bounds(drawing: &Drawing) -> Option<drawing_ir::BoundingBox> {
  let mut bounds = drawing_ir::BoundingBox::empty();
  for sheet in &drawing.sheets {
    if let Some(sheet_bounds) = sheet.bounds {
      bounds.include_point(Point::new(sheet_bounds.min_x, sheet_bounds.min_y));
      bounds.include_point(Point::new(sheet_bounds.max_x, sheet_bounds.max_y));
    } else {
      for entity in &sheet.entities {
        include_entity_bounds(entity, &mut bounds);
      }
    }
  }
  if bounds.is_valid() {
    Some(bounds)
  } else {
    None
  }
}

fn include_entity_bounds(entity: &Entity, bounds: &mut drawing_ir::BoundingBox) {
  match &entity.kind {
    EntityKind::Line { from, to } => {
      bounds.include_point(*from);
      bounds.include_point(*to);
    }
    EntityKind::Polyline(polyline) => {
      for point in &polyline.points {
        bounds.include_point(*point);
      }
    }
    EntityKind::Path(path) => {
      for segment in &path.segments {
        if let drawing_ir::PathSegment::MoveTo { to } | drawing_ir::PathSegment::LineTo { to } =
          segment
        {
          bounds.include_point(*to);
        }
      }
    }
    EntityKind::Rectangle {
      top_left,
      bottom_right,
    } => {
      bounds.include_point(*top_left);
      bounds.include_point(*bottom_right);
    }
    EntityKind::Arc(arc) => {
      for point in arc.sample_points(16) {
        bounds.include_point(point);
      }
    }
    EntityKind::Circle { center, radius } => {
      bounds.include_point(Point::new(center.x - radius, center.y - radius));
      bounds.include_point(Point::new(center.x + radius, center.y + radius));
    }
    EntityKind::Dimension(kind) => match kind {
      DimensionKind::Linear { from, to, .. } => {
        bounds.include_point(*from);
        bounds.include_point(*to);
      }
      DimensionKind::Radial { center, radius, .. } => {
        bounds.include_point(*center);
        bounds.include_point(Point::new(center.x + radius, center.y + radius));
      }
    },
    EntityKind::Text(text) => {
      bounds.include_point(text.position);
    }
  }
}

fn map_entity(entity: &Entity) -> Option<DxfEntity> {
  let specific = match &entity.kind {
    EntityKind::Line { from, to } => {
      let mut line = Line::default();
      line.p1 = dxf_point(from);
      line.p2 = dxf_point(to);
      EntityType::Line(line)
    }
    EntityKind::Polyline(polyline) => {
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
    EntityKind::Rectangle {
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
    EntityKind::Arc(arc) => EntityType::Arc(map_arc(arc)),
    EntityKind::Circle { center, radius } => {
      let mut circle = Circle::default();
      circle.center = dxf_point(center);
      circle.radius = *radius;
      EntityType::Circle(circle)
    }
    EntityKind::Text(text) => {
      let mut dxf_text = Text::default();
      dxf_text.location = dxf_point(&text.position);
      dxf_text.value = text.text.clone();
      dxf_text.text_height = text.font_size.max(0.1);
      dxf_text.rotation = text.rotation_deg;
      EntityType::Text(dxf_text)
    }
    EntityKind::Path(path) => {
      let lw = path_to_lwpolyline(path)?;
      EntityType::LwPolyline(lw)
    }
    EntityKind::Dimension(kind) => return map_dimension(kind, entity),
  };

  Some(DxfEntity::new(specific))
}

fn map_arc(arc: &ArcSegment) -> Arc {
  Arc::new(
    dxf_point(&arc.center),
    arc.radius,
    arc.start_angle.to_degrees(),
    arc.end_angle.to_degrees(),
  )
}

fn map_dimension(kind: &DimensionKind, entity: &Entity) -> Option<DxfEntity> {
  match kind {
    DimensionKind::Linear { from, to, text, .. } => {
      let mut line = Line::default();
      line.p1 = dxf_point(from);
      line.p2 = dxf_point(to);
      let mut dxf_entity = DxfEntity::new(EntityType::Line(line));
      apply_entity_style(&mut dxf_entity, entity);
      if let Some(label) = text {
        let mut dxf_text = Text::default();
        dxf_text.location = dxf_point(&Point::new((from.x + to.x) * 0.5, (from.y + to.y) * 0.5));
        dxf_text.value = label.clone();
        dxf_text.text_height = 2.5;
        return Some(DxfEntity::new(EntityType::Text(dxf_text)));
      }
      Some(dxf_entity)
    }
    DimensionKind::Radial {
      center,
      radius,
      text,
    } => {
      let mut circle = Circle::default();
      circle.center = dxf_point(center);
      circle.radius = *radius;
      let mut dxf_entity = DxfEntity::new(EntityType::Circle(circle));
      apply_entity_style(&mut dxf_entity, entity);
      if let Some(label) = text {
        let mut dxf_text = Text::default();
        dxf_text.location = dxf_point(&Point::new(center.x + radius, center.y));
        dxf_text.value = label.clone();
        dxf_text.text_height = 2.5;
        return Some(DxfEntity::new(EntityType::Text(dxf_text)));
      }
      Some(dxf_entity)
    }
  }
}

fn append_cam_entities(dxf: &mut DxfDrawing, cam: &drawing_ir::CamProgram) {
  for (index, tool) in cam.tools.iter().enumerate() {
    let mut text = Text::default();
    text.location = DxfPoint::new(0.0, -(index as f64 + 1.0) * 5.0, 0.0);
    text.value = format!(
      "{} {} {} {}",
      tool.kind,
      tool.size,
      tool.size2.unwrap_or(0.0),
      tool.comment.as_deref().unwrap_or("")
    );
    text.text_height = 2.5;
    let mut entity = DxfEntity::new(EntityType::Text(text));
    entity.common.layer = "TOOLS".to_string();
    dxf.add_entity(entity);
  }

  for operation in &cam.operations {
    match operation {
      CamOperation::Online { raw } => {
        if let Some(points) = extract_cam_points(raw) {
          add_cam_polyline(dxf, "CUT", &points);
        }
      }
      CamOperation::Single { position, .. } => {
        if let Some(point) = position {
          add_cam_point_marker(dxf, "PUNCH", *point);
        }
      }
      CamOperation::OnArc { raw } => {
        if let Some(points) = extract_cam_points(raw) {
          add_cam_polyline(dxf, "CUT", &points);
        }
      }
    }
  }
}

fn extract_cam_points(raw: &[String]) -> Option<Vec<Point>> {
  for line in raw {
    let values = parse_float_tokens(line);
    if values.len() >= 8 {
      let x1 = values[4];
      let y1 = values[5];
      let x2 = values[6];
      let y2 = values[7];
      if x1.is_finite() && y1.is_finite() && x2.is_finite() && y2.is_finite() {
        return Some(vec![Point::new(x1, y1), Point::new(x2, y2)]);
      }
    }
  }
  None
}

fn parse_float_tokens(line: &str) -> Vec<f64> {
  line
    .split_whitespace()
    .filter_map(|token| token.parse::<f64>().ok())
    .collect()
}

fn add_cam_polyline(dxf: &mut DxfDrawing, layer: &str, points: &[Point]) {
  if points.len() < 2 {
    return;
  }
  let mut lw = LwPolyline::default();
  lw.vertices = points
    .iter()
    .map(|point| LwPolylineVertex {
      x: point.x,
      y: point.y,
      ..Default::default()
    })
    .collect();
  let mut entity = DxfEntity::new(EntityType::LwPolyline(lw));
  entity.common.layer = layer.to_string();
  entity.common.color = Color::from_index(1);
  dxf.add_entity(entity);
}

fn add_cam_point_marker(dxf: &mut DxfDrawing, layer: &str, point: Point) {
  let mut circle = Circle::default();
  circle.center = dxf_point(&point);
  circle.radius = 1.0;
  let mut entity = DxfEntity::new(EntityType::Circle(circle));
  entity.common.layer = layer.to_string();
  entity.common.color = Color::from_index(3);
  dxf.add_entity(entity);
}

fn apply_entity_style(dxf_entity: &mut DxfEntity, entity: &Entity) {
  if let Some(layer) = &entity.layer {
    dxf_entity.common.layer = layer.clone();
  }
  if let Some(stroke) = &entity.style.stroke {
    if let Some(aci) = stroke_aci(stroke.color.r, stroke.color.g, stroke.color.b) {
      dxf_entity.common.color = Color::from_index(aci);
    }
  }
}

fn stroke_aci(r: u8, g: u8, b: u8) -> Option<u8> {
  match (r, g, b) {
    (255, 0, 0) => Some(1),
    (255, 255, 0) => Some(2),
    (0, 255, 0) => Some(3),
    (0, 255, 255) => Some(4),
    (0, 0, 255) => Some(5),
    (255, 0, 255) => Some(6),
    (255, 255, 255) | (0, 0, 0) => Some(7),
    _ => None,
  }
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

fn dxf_point(point: &Point) -> DxfPoint {
  DxfPoint::new(point.x, point.y, 0.0)
}
