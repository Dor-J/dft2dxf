//! CAM program metadata (cncKad punch/cut operations).

use serde::{Deserialize, Serialize};

use crate::geometry::Point;

/// One tool definition from a cncKad tool table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CamTool {
  /// Tool kind letter (`R`, `C`, etc.).
  pub kind: String,
  /// Tool size parameter.
  pub size: f64,
  /// Secondary size parameter when present.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub size2: Option<f64>,
  /// Tool comment from `TOOLCM`.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub comment: Option<String>,
}

/// One CAM operation (cut, punch, arc cut, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CamOperation {
  /// Linear cut path (`ONLINE`).
  Online {
    /// Raw operation lines for lossless round-trip.
    raw: Vec<String>,
  },
  /// Single punch hit (`SINGLE`).
  Single {
    /// Punch position when parsed.
    #[serde(skip_serializing_if = "Option::is_none")]
    position: Option<Point>,
    /// Tool index when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_index: Option<u32>,
    /// Raw operation lines.
    raw: Vec<String>,
  },
  /// Arc cut (`ONARC`).
  OnArc {
    /// Raw operation lines.
    raw: Vec<String>,
  },
}

/// Structured CAM program extracted from cncKad sections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CamProgram {
  /// Tool table entries.
  pub tools: Vec<CamTool>,
  /// Operations in document order.
  pub operations: Vec<CamOperation>,
}
