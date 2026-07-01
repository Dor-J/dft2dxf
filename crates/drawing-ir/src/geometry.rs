//! Basic geometry primitives.

use serde::Serialize;

/// A 2D point in drawing coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Point {
  /// X coordinate.
  pub x: f64,
  /// Y coordinate.
  pub y: f64,
}

impl Point {
  /// Creates a new point.
  #[must_use]
  pub const fn new(x: f64, y: f64) -> Self {
    Self { x, y }
  }
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BoundingBox {
  /// Minimum X.
  pub min_x: f64,
  /// Minimum Y.
  pub min_y: f64,
  /// Maximum X.
  pub max_x: f64,
  /// Maximum Y.
  pub max_y: f64,
}

impl BoundingBox {
  /// Empty bounding box.
  #[must_use]
  pub const fn empty() -> Self {
    Self {
      min_x: f64::INFINITY,
      min_y: f64::INFINITY,
      max_x: f64::NEG_INFINITY,
      max_y: f64::NEG_INFINITY,
    }
  }

  /// Expands the box to include a point.
  pub fn include_point(&mut self, point: Point) {
    self.min_x = self.min_x.min(point.x);
    self.min_y = self.min_y.min(point.y);
    self.max_x = self.max_x.max(point.x);
    self.max_y = self.max_y.max(point.y);
  }

  /// Returns whether the box contains any geometry.
  #[must_use]
  pub fn is_valid(&self) -> bool {
    self.min_x.is_finite()
      && self.min_y.is_finite()
      && self.max_x.is_finite()
      && self.max_y.is_finite()
  }
}

/// One segment in a path.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PathSegment {
  /// Move without drawing.
  MoveTo {
    /// Target point.
    to: Point,
  },
  /// Straight line.
  LineTo {
    /// Target point.
    to: Point,
  },
  /// Close subpath.
  Close,
}

/// A path made of segments.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Path {
  /// Path segments in order.
  pub segments: Vec<PathSegment>,
}

/// A polyline entity.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Polyline {
  /// Vertices in order.
  pub points: Vec<Point>,
  /// Whether the polyline is closed.
  pub closed: bool,
}

/// A circular arc segment.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ArcSegment {
  /// Center point.
  pub center: Point,
  /// Radius.
  pub radius: f64,
  /// Start angle in radians.
  pub start_angle: f64,
  /// End angle in radians.
  pub end_angle: f64,
}
