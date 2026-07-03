//! Inspect and validate API tests.

use dft2dxf_core::{
  inspect_bytes, validate_bytes, ConvertOptions, CoreError,
};
use dft2dxf_testkit::{
  build_minimal_dft, build_rectangle_emf, minimal_cnckad_dft, MinimalDftSpec,
};

fn synthetic_solid_edge_bytes() -> Vec<u8> {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 50, 50);
  build_minimal_dft(&path, &MinimalDftSpec::one_sheet("S1", emf)).unwrap();
  std::fs::read(path).unwrap()
}

#[test]
fn inspect_synthetic_solid_edge_dft() {
  let bytes = synthetic_solid_edge_bytes();
  let output = inspect_bytes(&bytes, &ConvertOptions::default()).unwrap();
  assert_eq!(output.format, "solid_edge");
  assert_eq!(output.sheets, 1);
  assert!(output.solid_edge.is_some());
}

#[test]
fn validate_synthetic_solid_edge_dft() {
  let bytes = synthetic_solid_edge_bytes();
  let output = validate_bytes(&bytes, &ConvertOptions::default()).unwrap();
  assert_eq!(output.status, "ok");
  assert_eq!(output.sheets, 1);
}

#[test]
fn inspect_cnckad_text_dft() {
  let bytes = minimal_cnckad_dft().into_bytes();
  let output = inspect_bytes(&bytes, &ConvertOptions::default()).unwrap();
  assert_eq!(output.format, "cnckad");
  assert!(output.entities > 0);
  assert_eq!(output.sheets, 1);
  assert!(output.solid_edge.is_none());
}

#[test]
fn validate_cnckad_text_dft() {
  let bytes = minimal_cnckad_dft().into_bytes();
  let output = validate_bytes(&bytes, &ConvertOptions::default()).unwrap();
  assert_eq!(output.status, "ok");
  assert_eq!(output.sheets, 1);
}

#[test]
fn inspect_rejects_unsupported_bytes() {
  let err = inspect_bytes(b"not a dft", &ConvertOptions::default()).unwrap_err();
  assert!(matches!(err, CoreError::UnsupportedFormat));
}

#[test]
fn validate_rejects_unsupported_bytes() {
  let err = validate_bytes(b"not a dft", &ConvertOptions::default()).unwrap_err();
  assert!(matches!(err, CoreError::UnsupportedFormat));
}
