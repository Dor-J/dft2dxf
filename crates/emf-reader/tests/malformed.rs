//! Malformed EMF parser and replay tests.

use emf_reader::{EmfDocument, EmfError, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE};

#[test]
fn rejects_emf_without_eof() {
  let mut data = vec![0u8; 88];
  data[0..4].copy_from_slice(&1u32.to_le_bytes());
  data[4..8].copy_from_slice(&88u32.to_le_bytes());
  data[40..44].copy_from_slice(&0x0000_464Du32.to_le_bytes());
  let err =
    EmfDocument::parse(&data, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap_err();
  assert!(matches!(err, EmfError::MissingEof));
}

#[test]
fn rejects_misaligned_record_size() {
  let mut data = vec![0u8; 88];
  data[0..4].copy_from_slice(&1u32.to_le_bytes());
  data[4..8].copy_from_slice(&87u32.to_le_bytes());
  data[40..44].copy_from_slice(&0x0000_464Du32.to_le_bytes());
  let err =
    EmfDocument::parse(&data, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap_err();
  assert!(matches!(err, EmfError::InvalidFormat { .. }));
}

#[test]
fn rejects_truncated_record_header() {
  let data = vec![0x01, 0x00, 0x00, 0x00];
  let err =
    EmfDocument::parse(&data, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap_err();
  assert!(matches!(err, EmfError::InvalidFormat { .. }));
}

#[test]
fn reports_unsupported_records_in_replay() {
  let emf = dft2dxf_testkit::build_rectangle_emf(0, 0, 10, 10);
  let doc = EmfDocument::parse(&emf, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE).unwrap();
  let drawing = emf_reader::replay_to_drawing(&doc, Some(1), None, None, None);
  assert!(!drawing.sheets.is_empty());
}
