//! Drawing-level metadata from source documents.

use serde::{Deserialize, Serialize};

/// Linear drawing units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaperUnit {
  /// Unitless coordinates.
  Unitless,
  /// Millimeters.
  Millimeters,
  /// Inches.
  Inches,
}

impl Default for PaperUnit {
  fn default() -> Self {
    Self::Millimeters
  }
}

/// Document-level metadata attached to a [`Drawing`](crate::Drawing).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DrawingMetadata {
  /// Part or drawing name.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub part_name: Option<String>,
  /// Customer or author name when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub customer: Option<String>,
  /// Material designation.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub material: Option<String>,
  /// Sheet thickness in drawing units.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub thickness: Option<f64>,
  /// Bend K-factor when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub k_factor: Option<f64>,
  /// Drawing scale factor when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub scale: Option<f64>,
  /// Linear units for coordinates.
  pub units: PaperUnit,
}

impl DrawingMetadata {
  /// Returns true when no metadata fields are populated.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.part_name.is_none()
      && self.customer.is_none()
      && self.material.is_none()
      && self.thickness.is_none()
      && self.k_factor.is_none()
      && self.scale.is_none()
      && self.units == PaperUnit::default()
  }
}
