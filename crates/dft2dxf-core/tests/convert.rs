//! Core conversion tests.

use dft2dxf_core::{convert_bytes, ConvertOptions};
use dft2dxf_testkit::{build_minimal_dft, build_rectangle_emf, MinimalDftSpec};

#[test]
fn converts_synthetic_solid_edge_dft_to_dxf_bytes() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 50, 50);
  build_minimal_dft(&path, &MinimalDftSpec::one_sheet("S1", emf)).unwrap();
  let bytes = std::fs::read(path).unwrap();
  let output = convert_bytes(&bytes, &ConvertOptions::default()).unwrap();
  assert!(!output.dxf.is_empty());
  let text = String::from_utf8_lossy(&output.dxf);
  assert!(text.contains("ENTITIES"));
}
