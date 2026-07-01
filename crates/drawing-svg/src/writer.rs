//! SVG writer implementation.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path as FsPath;

use drawing_ir::{BoundingBox, EntityKind, PathSegment, Point};
use svg::node::element::path::Data;
use svg::node::element::{
  Circle, Group, Line, Path as SvgPath, Polygon, Polyline, Text as SvgText,
};
use svg::node::{Text as TextNode, Value};
use svg::Document;

use crate::error::SvgResult;

/// Padding around computed bounds in drawing units.
const VIEWBOX_PADDING: f64 = 5.0;

/// Serializes a drawing to a deterministic SVG string.
pub fn write_drawing_to_string(drawing: &drawing_ir::Drawing) -> SvgResult<String> {
  let bounds = compute_drawing_bounds(drawing);
  let (view_box, flip_offset) = view_box_and_flip(&bounds);

  let mut doc = Document::new().set("viewBox", view_box);
  for sheet in &drawing.sheets {
    let mut sheet_group = Group::new();
    if let Some(name) = &sheet.name {
      sheet_group = sheet_group.set("data-sheet-name", name.as_str());
    }

    let layer_groups = group_entities_by_layer(&sheet.entities, flip_offset);
    for (layer_name, entities) in layer_groups {
      let mut layer_group = Group::new().set("data-layer", layer_name.as_str());
      for entity in entities {
        layer_group = layer_group.add(render_entity(entity, flip_offset));
      }
      sheet_group = sheet_group.add(layer_group);
    }

    doc = doc.add(sheet_group);
  }
  Ok(doc.to_string())
}

/// Writes a drawing to a file path.
pub fn write_drawing_to_file(drawing: &drawing_ir::Drawing, path: &FsPath) -> SvgResult<()> {
  if let Some(parent) = path.parent() {
    if !parent.as_os_str().is_empty() {
      std::fs::create_dir_all(parent)?;
    }
  }
  let content = write_drawing_to_string(drawing)?;
  let mut file = std::fs::File::create(path)?;
  file.write_all(content.as_bytes())?;
  Ok(())
}

fn compute_drawing_bounds(drawing: &drawing_ir::Drawing) -> BoundingBox {
  let mut bounds = BoundingBox::empty();
  for sheet in &drawing.sheets {
    if let Some(sheet_bounds) = sheet.bounds {
      bounds.include_point(Point::new(sheet_bounds.min_x, sheet_bounds.min_y));
      bounds.include_point(Point::new(sheet_bounds.max_x, sheet_bounds.max_y));
      continue;
    }
    for entity in &sheet.entities {
      include_entity_bounds(entity, &mut bounds);
    }
  }
  bounds
}

fn include_entity_bounds(entity: &drawing_ir::Entity, bounds: &mut BoundingBox) {
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
        if let PathSegment::MoveTo { to } | PathSegment::LineTo { to } = segment {
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
      drawing_ir::DimensionKind::Linear { from, to, .. } => {
        bounds.include_point(*from);
        bounds.include_point(*to);
      }
      drawing_ir::DimensionKind::Radial { center, radius, .. } => {
        bounds.include_point(*center);
        bounds.include_point(Point::new(center.x + radius, center.y + radius));
      }
    },
    EntityKind::Text(text) => {
      bounds.include_point(text.position);
    }
  }
}

fn view_box_and_flip(bounds: &BoundingBox) -> (String, f64) {
  if !bounds.is_valid() {
    return ("0 0 100 100".to_string(), 0.0);
  }
  let min_x = bounds.min_x - VIEWBOX_PADDING;
  let min_y = bounds.min_y - VIEWBOX_PADDING;
  let width = (bounds.max_x - bounds.min_x) + VIEWBOX_PADDING * 2.0;
  let height = (bounds.max_y - bounds.min_y) + VIEWBOX_PADDING * 2.0;
  let flip_offset = bounds.min_y + bounds.max_y;
  (format!("{min_x} {min_y} {width} {height}"), flip_offset)
}

fn group_entities_by_layer<'a>(
  entities: &'a [drawing_ir::Entity],
  _flip_offset: f64,
) -> BTreeMap<String, Vec<&'a drawing_ir::Entity>> {
  let mut grouped: BTreeMap<String, Vec<&drawing_ir::Entity>> = BTreeMap::new();
  for entity in entities {
    let layer = entity.layer.as_deref().unwrap_or("0").to_string();
    grouped.entry(layer).or_default().push(entity);
  }
  grouped
}

fn flip_y(point: Point, flip_offset: f64) -> Point {
  Point::new(point.x, flip_offset - point.y)
}

fn render_entity(entity: &drawing_ir::Entity, flip_offset: f64) -> Group {
  let stroke = entity
    .style
    .stroke
    .as_ref()
    .map(|value| {
      format!(
        "#{:02x}{:02x}{:02x}",
        value.color.r, value.color.g, value.color.b
      )
    })
    .unwrap_or_else(|| "#000000".to_string());
  let stroke_width = entity
    .style
    .stroke
    .as_ref()
    .map(|value| value.width)
    .unwrap_or(1.0);

  match &entity.kind {
    EntityKind::Line { from, to } => {
      let from = flip_y(*from, flip_offset);
      let to = flip_y(*to, flip_offset);
      Group::new().add(
        Line::new()
          .set("x1", fmt(from.x))
          .set("y1", fmt(from.y))
          .set("x2", fmt(to.x))
          .set("y2", fmt(to.y))
          .set("stroke", stroke)
          .set("stroke-width", fmt(stroke_width))
          .set("fill", "none"),
      )
    }
    EntityKind::Polyline(polyline) => {
      let points = polyline
        .points
        .iter()
        .map(|point| {
          let flipped = flip_y(*point, flip_offset);
          format!("{},{}", fmt(flipped.x), fmt(flipped.y))
        })
        .collect::<Vec<_>>()
        .join(" ");
      if polyline.closed {
        Group::new().add(
          Polygon::new()
            .set("points", points)
            .set("stroke", stroke)
            .set("stroke-width", fmt(stroke_width))
            .set("fill", "none"),
        )
      } else {
        Group::new().add(
          Polyline::new()
            .set("points", points)
            .set("stroke", stroke)
            .set("stroke-width", fmt(stroke_width))
            .set("fill", "none"),
        )
      }
    }
    EntityKind::Path(path) => Group::new().add(
      SvgPath::new()
        .set("d", path_data(path, flip_offset))
        .set("stroke", stroke)
        .set("stroke-width", fmt(stroke_width))
        .set("fill", "none"),
    ),
    EntityKind::Rectangle {
      top_left,
      bottom_right,
    } => {
      let tl = flip_y(*top_left, flip_offset);
      let br = flip_y(*bottom_right, flip_offset);
      Group::new().add(
        SvgPath::new()
          .set(
            "d",
            format!(
              "M {} {} L {} {} L {} {} L {} {} Z",
              fmt(tl.x),
              fmt(tl.y),
              fmt(br.x),
              fmt(tl.y),
              fmt(br.x),
              fmt(br.y),
              fmt(tl.x),
              fmt(br.y)
            ),
          )
          .set("stroke", stroke)
          .set("stroke-width", fmt(stroke_width))
          .set("fill", "none"),
      )
    }
    EntityKind::Arc(arc) => {
      let start = flip_y(arc.point_at_angle(arc.start_angle), flip_offset);
      let end = flip_y(arc.point_at_angle(arc.end_angle), flip_offset);
      let sweep = arc.end_angle - arc.start_angle;
      let large_arc = if sweep.abs() > std::f64::consts::PI {
        1
      } else {
        0
      };
      let sweep_flag = if sweep >= 0.0 { 1 } else { 0 };
      Group::new().add(
        SvgPath::new()
          .set(
            "d",
            format!(
              "M {} {} A {} {} 0 {} {} {} {}",
              fmt(start.x),
              fmt(start.y),
              fmt(arc.radius),
              fmt(arc.radius),
              large_arc,
              sweep_flag,
              fmt(end.x),
              fmt(end.y)
            ),
          )
          .set("stroke", stroke)
          .set("stroke-width", fmt(stroke_width))
          .set("fill", "none"),
      )
    }
    EntityKind::Circle { center, radius } => {
      let center = flip_y(*center, flip_offset);
      Group::new().add(
        Circle::new()
          .set("cx", fmt(center.x))
          .set("cy", fmt(center.y))
          .set("r", fmt(*radius))
          .set("stroke", stroke)
          .set("stroke-width", fmt(stroke_width))
          .set("fill", "none"),
      )
    }
    EntityKind::Text(text) => {
      let position = flip_y(text.position, flip_offset);
      Group::new().add(
        SvgText::new("")
          .set("x", fmt(position.x))
          .set("y", fmt(position.y))
          .set("fill", stroke)
          .set("font-size", fmt(text.font_size.max(0.1)))
          .set(
            "transform",
            format!(
              "rotate({} {} {})",
              text.rotation_deg,
              fmt(position.x),
              fmt(position.y)
            ),
          )
          .add(TextNode::new(text.text.as_str())),
      )
    }
    EntityKind::Dimension(kind) => match kind {
      drawing_ir::DimensionKind::Linear { from, to, .. } => {
        let from = flip_y(*from, flip_offset);
        let to = flip_y(*to, flip_offset);
        Group::new().add(
          Line::new()
            .set("x1", fmt(from.x))
            .set("y1", fmt(from.y))
            .set("x2", fmt(to.x))
            .set("y2", fmt(to.y))
            .set("stroke", stroke)
            .set("stroke-width", fmt(stroke_width))
            .set("fill", "none"),
        )
      }
      drawing_ir::DimensionKind::Radial { center, radius, .. } => {
        let center = flip_y(*center, flip_offset);
        Group::new().add(
          Circle::new()
            .set("cx", fmt(center.x))
            .set("cy", fmt(center.y))
            .set("r", fmt(*radius))
            .set("stroke", stroke)
            .set("stroke-width", fmt(stroke_width))
            .set("fill", "none"),
        )
      }
    },
  }
}

fn path_data(path: &drawing_ir::Path, flip_offset: f64) -> String {
  let mut data = Data::new();
  for segment in &path.segments {
    match segment {
      PathSegment::MoveTo { to } => {
        let point = flip_y(*to, flip_offset);
        data = data.move_to((point.x, point.y));
      }
      PathSegment::LineTo { to } => {
        let point = flip_y(*to, flip_offset);
        data = data.line_to((point.x, point.y));
      }
      PathSegment::Close => {
        data = data.close();
      }
    }
  }
  Value::from(data).to_string()
}

fn fmt(value: f64) -> String {
  if value.fract() == 0.0 {
    format!("{value:.0}")
  } else {
    format!("{value:.3}")
  }
}
