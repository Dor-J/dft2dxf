//! Malformed metadata bytes for negative tests.

/// Metadata stream truncated before sheet fields.
#[must_use]
pub fn too_short_metadata() -> Vec<u8> {
  vec![0x01, 0x00, 0x00, 0x00]
}

/// Metadata declaring a negative sheet count.
#[must_use]
pub fn negative_sheet_count_metadata() -> Vec<u8> {
  let mut data = Vec::new();
  data.extend_from_slice(&1u32.to_le_bytes());
  data.extend_from_slice(&(-1i32).to_le_bytes());
  data.extend_from_slice(&0i32.to_le_bytes());
  data.extend_from_slice(&1u32.to_le_bytes());
  data.extend_from_slice(&0x003Du16.to_le_bytes());
  data
}

/// Metadata declaring more sheets than the configured limit allows.
#[must_use]
pub fn excessive_sheet_count_metadata(count: i32) -> Vec<u8> {
  let mut data = Vec::new();
  data.extend_from_slice(&1u32.to_le_bytes());
  data.extend_from_slice(&count.to_le_bytes());
  data.extend_from_slice(&0i32.to_le_bytes());
  data.extend_from_slice(&1u32.to_le_bytes());
  data.extend_from_slice(&0x003Du16.to_le_bytes());
  data
}

/// Invalid zlib payload for decompression tests.
#[must_use]
pub fn invalid_zlib_payload() -> Vec<u8> {
  vec![0xFF, 0xFE, 0xFD, 0xFC, 0xFB]
}
