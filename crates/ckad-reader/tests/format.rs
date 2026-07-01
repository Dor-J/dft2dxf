//! Format detection tests.

use ckad_reader::{detect_format, DftContainerFormat};

#[test]
fn detects_utf16_le_cnckad_header() {
  let bytes = dft2dxf_testkit::minimal_cnckad_dft_utf16_le();
  assert_eq!(detect_format(&bytes), Some(DftContainerFormat::CncKad));
}

#[test]
fn detects_ascii_cnckad_header() {
  let bytes = dft2dxf_testkit::minimal_cnckad_dft().into_bytes();
  assert_eq!(detect_format(&bytes), Some(DftContainerFormat::CncKad));
}
