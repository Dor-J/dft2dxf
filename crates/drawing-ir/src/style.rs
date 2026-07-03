//! Stroke and fill styles.

use serde::{Deserialize, Serialize};

/// RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
  /// Red channel 0-255.
  pub r: u8,
  /// Green channel 0-255.
  pub g: u8,
  /// Blue channel 0-255.
  pub b: u8,
  /// Alpha channel 0-255.
  pub a: u8,
}

impl Color {
  /// Creates a new opaque color.
  #[must_use]
  pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
    Self { r, g, b, a: 255 }
  }

  /// Black color.
  #[must_use]
  pub const fn black() -> Self {
    Self::rgb(0, 0, 0)
  }
}

/// Stroke style for vector primitives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrokeStyle {
  /// Stroke color.
  pub color: Color,
  /// Stroke width in drawing units.
  pub width: f64,
}

impl Default for StrokeStyle {
  fn default() -> Self {
    Self {
      color: Color::black(),
      width: 1.0,
    }
  }
}

/// Fill style for closed geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillStyle {
  /// Fill color.
  pub color: Color,
}

/// Combined style for an entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Style {
  /// Optional stroke.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stroke: Option<StrokeStyle>,
  /// Optional fill.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub fill: Option<FillStyle>,
}

impl Default for Style {
  fn default() -> Self {
    Self {
      stroke: Some(StrokeStyle::default()),
      fill: None,
    }
  }
}
