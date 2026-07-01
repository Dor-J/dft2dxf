//! SVG writer implementation.

use std::io::Write;
use std::path::Path;

use drawing_ir::{
  EntityKind, PathSegment, Point,
};
use svg::node::element::path::Data;
use svg::node::element::{Group, Line, Path, Polygon, Polyline, Text};
use svg::node::Text as TextNode;
use svg::Document;

use crate::error::SvgResult;

/// Serializes a drawing to a deterministic SVG string.
pub fn write_drawing_to_string(drawing: &drawing_ir::Drawing) -> SvgResult<String> {
  let mut doc = Document::new().set("viewBox", "0 0 1000 1000");
  for sheet in &drawing.sheets {
    let mut group = Group::new();
    if let Some(name) = &sheet.name {
      group = group.set("data-sheet-name", name.as_str());
    }
    for entity in &sheet.entities {
      group = group.add(render_entity(entity));
    }
    doc = doc.add(group);
  }
  Ok(doc.to_string())
}

/// Writes a drawing to a file path.
pub fn write_drawing_to_file(drawing: &drawing_ir::Drawing, path: &Path) -> SvgResult<()> {
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

fn render_entity(entity: &drawing_ir::Entity) -> svg::node::element::Group {
  let stroke = entity
    .style
    .stroke
    .as_ref()
    .map(|value| format!("#{:02x}{:02x}{:02x}", value.color.r, value.color.g, value.color.b))
    .unwrap_or_else(|| "#000000".to_string());
  let stroke_width = entity
    .style
    .stroke
    .as_ref()
    .map(|value| value.width)
    .unwrap_or(1.0);

  match &entity.kind {
    EntityKind::Line { from, to } => Group::new().add(
      Line::new()
        .set("x1", fmt(from.x))
        .set("y1", fmt(from.y))
        .set("x2", fmt(to.x))
        .set("y2", fmt(to.y))
        .set("stroke", stroke)
        .set("stroke-width", fmt(stroke_width))
        .set("fill", "none"),
    ),
    EntityKind::Polyline(polyline) => {
      let points = polyline
        .points
        .iter()
        .map(|point| format!("{},{}", fmt(point.x), fmt(point.y)))
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
      Path::new()
        .set("d", path_data(path))
        .set("stroke", stroke)
        .set("stroke-width", fmt(stroke_width))
        .set("fill", "none"),
    ),
    EntityKind::Rectangle {
      top_left,
      bottom_right,
    } => Group::new().add(
      Path::new()
        .set(
          "d",
          format!(
            "M {} {} L {} {} L {} {} L {} {} Z",
            fmt(top_left.x),
            fmt(top_left.y),
            fmt(bottom_right.x),
            fmt(top_left.y),
            fmt(bottom_right.x),
            fmt(bottom_right.y),
            fmt(top_left.x),
            fmt(bottom_right.y)
          ),
        )
        .set("stroke", stroke)
        .set("stroke-width", fmt(stroke_width))
        .set("fill", "none"),
    ),
    EntityKind::Text(text) => Group::new().add(
      Text::new()
        .set("x", fmt(text.position.x))
        .set("y", fmt(text.position.y))
        .set("fill", stroke)
        .add(TextNode::new(text.text.as_str())),
    ),
    EntityKind::Arc(_) => Group::new(),
  }
}

fn path_data(path: &drawing_ir::Path) -> String {
  let mut data = Data::new();
  for segment in &path.segments {
    match segment {
      PathSegment::MoveTo { to } => {
        data = data.move_to((to.x, to.y));
      }
      PathSegment::LineTo { to } => {
        data = data.line_to((to.x, to.y));
      }
      PathSegment::Close => {
        data = data.close();
      }
    }
  }
  data.to_string()
}

fn fmt(value: f64) -> String {
  if value.fract() == 0.0 {
    format!("{value:.0}")
  } else {
    format!("{value:.3}")
  }
}

#[allow(dead_code)]
fn point_pair(point: Point) -> (f64, f64) {
  (point.x, point.y)
}
