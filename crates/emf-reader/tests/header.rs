//! EMF header parsing tests.

use dft2dxf_testkit::{build_emf_invalid_bounds, build_emf_wrong_n_bytes, build_rectangle_emf};
use emf_reader::{
  EmfDocument, EmfError, EmfHeader, EMF_SIGNATURE, DEFAULT_MAX_RECORD_COUNT,
  DEFAULT_MAX_RECORD_SIZE,
};

#[test]
fn parses_valid_header_from_testkit_emf() {
  let emf = build_rectangle_emf(0, 0, 100, 50);
  let doc = EmfDocument::parse(&emf, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap();
  assert_eq!(doc.header.signature, EMF_SIGNATURE);
  assert_eq!(doc.header.n_bytes, u32::try_from(emf.len()).unwrap());
  assert!(doc.header.bounds.is_valid());
}

#[test]
fn rejects_wrong_n_bytes() {
  let emf = build_emf_wrong_n_bytes();
  let err =
    EmfDocument::parse(&emf, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap_err();
  assert!(matches!(err, EmfError::InvalidFormat { .. }));
}

#[test]
fn rejects_invalid_bounds() {
  let emf = build_emf_invalid_bounds();
  let err =
    EmfDocument::parse(&emf, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap_err();
  assert!(matches!(err, EmfError::InvalidFormat { .. }));
}

#[test]
fn header_record_count_mismatch_returns_message() {
  let emf = build_rectangle_emf(0, 0, 10, 10);
  let doc = EmfDocument::parse(&emf, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap();
  assert!(doc.header.record_count_mismatch(999).is_some());
  assert!(doc.header.record_count_mismatch(u32::try_from(doc.records.len()).unwrap()).is_none());
}

#[test]
fn rejects_missing_header_record_type() {
  let mut data = vec![0u8; 88];
  data[0..4].copy_from_slice(&14u32.to_le_bytes()); // EOF as first record
  data[4..8].copy_from_slice(&88u32.to_le_bytes());
  let err =
    EmfDocument::parse(&data, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap_err();
  assert!(matches!(err, EmfError::MissingEof | EmfError::InvalidFormat { .. }));
}

#[test]
fn emf_header_parse_standalone() {
  let emf = build_rectangle_emf(0, 0, 10, 10);
  let header = EmfHeader::parse(&emf[..88]).unwrap();
  assert_eq!(header.n_bytes, u32::try_from(emf.len()).unwrap());
}
