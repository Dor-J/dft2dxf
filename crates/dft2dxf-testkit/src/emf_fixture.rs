//! Build minimal synthetic EMF files for tests.

/// EMF signature.
const EMF_SIGNATURE: u32 = 0x0000_464D;
/// `EMR_HEADER`
const EMR_HEADER: u32 = 1;
/// `EMR_EOF`
const EMR_EOF: u32 = 14;
/// `EMR_RECTANGLE`
const EMR_RECTANGLE: u32 = 42;
/// `EMR_LINETO` (unused directly here)
const _EMR_LINETO: u32 = 54;

/// Builds a minimal valid EMF containing one rectangle record.
pub fn build_rectangle_emf(left: i32, top: i32, right: i32, bottom: i32) -> Vec<u8> {
  let header_size = 88u32;
  let rect_size = 24u32;
  let eof_size = 20u32;
  let file_size = header_size + rect_size + eof_size;

  let mut data = Vec::new();
  append_record_header(&mut data, EMR_HEADER, header_size);
  // rclBounds + rclFrame placeholders (32 bytes)
  data.extend_from_slice(&[0u8; 32]);
  // dSignature = 0x464D (" EMF" little-endian)
  data.extend_from_slice(&EMF_SIGNATURE.to_le_bytes());
  // nVersion and remaining header fields
  let remaining = header_size as usize - data.len();
  data.extend_from_slice(&vec![0u8; remaining]);
  // nBytes is typically at offset 48 in EMR_HEADER
  if data.len() >= 52 {
    data[48..52].copy_from_slice(&file_size.to_le_bytes());
  }

  append_record_header(&mut data, EMR_RECTANGLE, rect_size);
  data.extend_from_slice(&left.to_le_bytes());
  data.extend_from_slice(&top.to_le_bytes());
  data.extend_from_slice(&right.to_le_bytes());
  data.extend_from_slice(&bottom.to_le_bytes());

  append_record_header(&mut data, EMR_EOF, eof_size);
  data.extend_from_slice(&[0u8; eof_size as usize - 8]);
  data
}

/// Builds a minimal valid EMF containing one polyline-like rectangle.
pub fn build_line_emf(x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<u8> {
  build_rectangle_emf(x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2))
}

fn append_record_header(buf: &mut Vec<u8>, record_type: u32, size: u32) {
  buf.extend_from_slice(&record_type.to_le_bytes());
  buf.extend_from_slice(&size.to_le_bytes());
}

/// Validates that bytes contain a valid EMF signature in the header record.
pub fn is_emf(data: &[u8]) -> bool {
  data.len() >= 44
    && u32::from_le_bytes(data[0..4].try_into().unwrap()) == EMR_HEADER
    && u32::from_le_bytes(data[40..44].try_into().unwrap()) == EMF_SIGNATURE
}
