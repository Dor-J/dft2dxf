//! Drawing entities.

use serde::Serialize;

use crate::diagnostic::SourceProvenance;
use crate::geometry::{ArcSegment, Path, Point, Polyline};
use crate::style::Style;

#[allow(clippy::trivially_copy_pass_by_ref)] // serde `skip_serializing_if` requires `fn(&T) -> bool`
fn is_zero(value: &f64) -> bool {
  *value == 0.0
}

/// Linear or radial dimension annotation.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DimensionKind {
  /// Linear distance between two points.
  Linear {
    /// First anchor.
    from: Point,
    /// Second anchor.
    to: Point,
    /// Dimension line offset.
    offset: f64,
    /// Display text override.
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
  },
  /// Radial dimension from a center.
  Radial {
    /// Arc/circle center.
    center: Point,
    /// Radius.
    radius: f64,
    /// Display text override.
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
  },
}

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
  /// Rotation in degrees (counter-clockwise).
  #[serde(skip_serializing_if = "is_zero")]
  pub rotation_deg: f64,
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
  /// Full circle.
  Circle {
    /// Center point.
    center: Point,
    /// Radius.
    radius: f64,
  },
  /// Dimension annotation.
  Dimension(DimensionKind),
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
