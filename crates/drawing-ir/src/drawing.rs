//! Drawing and sheet containers.

use serde::Serialize;

use crate::cam::CamProgram;
use crate::diagnostic::Diagnostic;
use crate::entity::Entity;
use crate::geometry::{BoundingBox, Point};
use crate::metadata::DrawingMetadata;

/// One sheet/page in a drawing.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sheet {
  /// One-based sheet index when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub index: Option<u32>,
  /// Sheet name when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  /// Sheet width in drawing units when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub width: Option<f64>,
  /// Sheet height in drawing units when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub height: Option<f64>,
  /// Drawable entities.
  pub entities: Vec<Entity>,
  /// Computed bounds.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub bounds: Option<BoundingBox>,
}

impl Sheet {
  /// Recomputes the bounding box from entities.
  pub fn recompute_bounds(&mut self) {
    let mut bounds = BoundingBox::empty();
    for entity in &self.entities {
      include_entity_in_bounds(entity, &mut bounds);
    }
    self.bounds = if bounds.is_valid() {
      Some(bounds)
    } else {
      None
    };
  }
}

fn include_entity_in_bounds(entity: &Entity, bounds: &mut BoundingBox) {
  match &entity.kind {
    crate::entity::EntityKind::Line { from, to } => {
      bounds.include_point(*from);
      bounds.include_point(*to);
    }
    crate::entity::EntityKind::Polyline(polyline) => {
      for point in &polyline.points {
        bounds.include_point(*point);
      }
    }
    crate::entity::EntityKind::Path(path) => {
      for segment in &path.segments {
        if let crate::geometry::PathSegment::LineTo { to }
        | crate::geometry::PathSegment::MoveTo { to } = segment
        {
          bounds.include_point(*to);
        }
      }
    }
    crate::entity::EntityKind::Rectangle {
      top_left,
      bottom_right,
    } => {
      bounds.include_point(*top_left);
      bounds.include_point(*bottom_right);
    }
    crate::entity::EntityKind::Arc(arc) => {
      for point in arc.sample_points(16) {
        bounds.include_point(point);
      }
    }
    crate::entity::EntityKind::Circle { center, radius } => {
      bounds.include_point(Point::new(center.x - radius, center.y - radius));
      bounds.include_point(Point::new(center.x + radius, center.y + radius));
    }
    crate::entity::EntityKind::Dimension(kind) => match kind {
      crate::entity::DimensionKind::Linear { from, to, .. } => {
        bounds.include_point(*from);
        bounds.include_point(*to);
      }
      crate::entity::DimensionKind::Radial { center, radius, .. } => {
        bounds.include_point(*center);
        bounds.include_point(Point::new(center.x + radius, center.y + radius));
      }
    },
    crate::entity::EntityKind::Text(text) => {
      bounds.include_point(text.position);
    }
  }
}

/// A complete drawing with one or more sheets.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Drawing {
  /// Source file path when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source_path: Option<String>,
  /// Sheets in document order.
  pub sheets: Vec<Sheet>,
  /// Conversion diagnostics.
  pub diagnostics: Vec<Diagnostic>,
  /// Document metadata (part, material, thickness, etc.).
  #[serde(skip_serializing_if = "DrawingMetadata::is_empty")]
  pub metadata: DrawingMetadata,
  /// CAM program when extracted.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cam: Option<CamProgram>,
}

impl Drawing {
  /// Creates an empty drawing.
  #[must_use]
  pub fn new() -> Self {
    Self {
      source_path: None,
      sheets: Vec::new(),
      diagnostics: Vec::new(),
      metadata: DrawingMetadata::default(),
      cam: None,
    }
  }

  /// Adds a diagnostic.
  pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostics.push(diagnostic);
  }
}

impl Default for Drawing {
  fn default() -> Self {
    Self::new()
  }
}
