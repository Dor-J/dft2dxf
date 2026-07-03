//! Diagnostic helper tests.

use drawing_ir::{Diagnostic, DiagnosticSeverity, SourceProvenance};

#[test]
fn warning_diagnostic_has_expected_fields() {
  let diag = Diagnostic::warning("test.code", "test message");
  assert_eq!(diag.severity, DiagnosticSeverity::Warning);
  assert_eq!(diag.code, "test.code");
  assert_eq!(diag.message, "test message");
  assert!(diag.provenance.is_none());
}

#[test]
fn unsupported_record_diagnostic_includes_provenance() {
  let diag = Diagnostic::unsupported_record(0x1234_5678, 3);
  assert_eq!(diag.severity, DiagnosticSeverity::Warning);
  assert_eq!(diag.code, "emf.unsupported_record");
  let provenance = diag.provenance.expect("provenance");
  assert_eq!(
    provenance,
    SourceProvenance {
      emf_record_index: Some(3),
      emf_record_type: Some(0x1234_5678),
    }
  );
}

#[test]
fn unsupported_dxf_entity_diagnostic_formats_message() {
  let diag = Diagnostic::unsupported_dxf_entity("ellipse", "not mapped yet");
  assert_eq!(diag.code, "dxf.unsupported_entity");
  assert!(diag.message.contains("ellipse"));
  assert!(diag.message.contains("not mapped yet"));
}
