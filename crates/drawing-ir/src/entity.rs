//! Drawing entities.

use serde::Serialize;

use crate::diagnostic::SourceProvenance;
use crate::geometry::{ArcSegment, Path, Point, Polyline};
use crate::style::Style;

/// Text alignment placeholder for EMF text output.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TextRun {
  /// Anchor position.
  pub position: Point,
  /// Text content.
  pub text: String,
  /// Font family name when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub font_family: Option<String>,
  /// Font height in drawing units.
  pub font_size: f64,
  /// Visual style.
  pub style: Style,
  /// Source provenance.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub provenance: Option<SourceProvenance>,
}

/// Supported entity kinds in the drawing IR.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EntityKind {
  /// Line between two points.
  Line {
    /// Start point.
    from: Point,
    /// End point.
    to: Point,
  },
  /// Polyline.
  Polyline(Polyline),
  /// Path geometry.
  Path(Path),
  /// Rectangle defined by opposite corners.
  Rectangle {
    /// Top-left corner.
    top_left: Point,
    /// Bottom-right corner.
    bottom_right: Point,
  },
  /// Circular arc.
  Arc(ArcSegment),
  /// Text run.
  Text(TextRun),
}

/// One drawable entity on a sheet.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Entity {
  /// Layer name when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub layer: Option<String>,
  /// Visual style.
  pub style: Style,
  /// Geometry payload.
  pub kind: EntityKind,
  /// Source provenance.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub provenance: Option<SourceProvenance>,
}
