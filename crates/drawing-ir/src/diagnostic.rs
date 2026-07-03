//! Diagnostics and provenance metadata.

use serde::{Deserialize, Serialize};

/// Severity for a diagnostic message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
  /// Informational note.
  Info,
  /// Recoverable issue.
  Warning,
  /// Unsupported or invalid input.
  Error,
}

/// Source location for a converted primitive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceProvenance {
  /// EMF record index when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub emf_record_index: Option<u32>,
  /// EMF record type when known.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub emf_record_type: Option<u32>,
}

/// One diagnostic emitted during conversion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
  /// Severity level.
  pub severity: DiagnosticSeverity,
  /// Stable diagnostic code.
  pub code: String,
  /// Human-readable message.
  pub message: String,
  /// Optional provenance.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub provenance: Option<SourceProvenance>,
}

impl Diagnostic {
  /// Creates a warning diagnostic.
  #[must_use]
  pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
    Self {
      severity: DiagnosticSeverity::Warning,
      code: code.into(),
      message: message.into(),
      provenance: None,
    }
  }

  /// Creates an unsupported-record diagnostic.
  #[must_use]
  pub fn unsupported_record(record_type: u32, record_index: u32) -> Self {
    Self {
      severity: DiagnosticSeverity::Warning,
      code: "emf.unsupported_record".to_string(),
      message: format!("unsupported EMF record type 0x{record_type:08X}"),
      provenance: Some(SourceProvenance {
        emf_record_index: Some(record_index),
        emf_record_type: Some(record_type),
      }),
    }
  }

  /// Creates a diagnostic for a Drawing IR entity that cannot be exported to DXF.
  #[must_use]
  pub fn unsupported_dxf_entity(entity_kind: &str, message: impl Into<String>) -> Self {
    Self {
      severity: DiagnosticSeverity::Warning,
      code: "dxf.unsupported_entity".to_string(),
      message: format!("{entity_kind}: {}", message.into()),
      provenance: None,
    }
  }
}
