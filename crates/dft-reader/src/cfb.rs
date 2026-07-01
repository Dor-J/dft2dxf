//! Bounded binary parsing helpers.

use crate::error::{DftError, DftResult};

/// A byte slice cursor with bounds checking.
#[derive(Debug, Clone)]
pub(crate) struct ByteCursor<'a> {
  data: &'a [u8],
  offset: usize,
}

impl<'a> ByteCursor<'a> {
  /// Creates a new cursor over `data`.
  pub(crate) fn new(data: &'a [u8]) -> Self {
    Self { data, offset: 0 }
  }

  /// Returns remaining bytes.
  pub(crate) fn remaining(&self) -> &'a [u8] {
    &self.data[self.offset..]
  }

  /// Returns current offset.
  pub(crate) fn offset(&self) -> usize {
    self.offset
  }

  /// Returns whether the cursor is exhausted.
  pub(crate) fn is_empty(&self) -> bool {
    self.offset >= self.data.len()
  }

  /// Reads exactly `len` bytes.
  pub(crate) fn read_bytes(&mut self, len: usize, context: &str) -> DftResult<&'a [u8]> {
    let end = self
      .offset
      .checked_add(len)
      .ok_or_else(|| DftError::InvalidMetadata {
        context: context.to_string(),
        message: "offset overflow".to_string(),
      })?;
    if end > self.data.len() {
      return Err(DftError::InvalidMetadata {
        context: context.to_string(),
        message: format!("unexpected end of stream (need {len} bytes)"),
      });
    }
    let slice = &self.data[self.offset..end];
    self.offset = end;
    Ok(slice)
  }

  /// Reads a little-endian `u32`.
  pub(crate) fn read_u32_le(&mut self, context: &str) -> DftResult<u32> {
    let bytes = self.read_bytes(4, context)?;
    Ok(u32::from_le_bytes(bytes.try_into().expect("4 bytes")))
  }

  /// Reads a little-endian `i32`.
  pub(crate) fn read_i32_le(&mut self, context: &str) -> DftResult<i32> {
    let bytes = self.read_bytes(4, context)?;
    Ok(i32::from_le_bytes(bytes.try_into().expect("4 bytes")))
  }

  /// Reads a little-endian `u16`.
  pub(crate) fn read_u16_le(&mut self, context: &str) -> DftResult<u16> {
    let bytes = self.read_bytes(2, context)?;
    Ok(u16::from_le_bytes(bytes.try_into().expect("2 bytes")))
  }

  /// Reads a little-endian `f64`.
  pub(crate) fn read_f64_le(&mut self, context: &str) -> DftResult<f64> {
    let bytes = self.read_bytes(8, context)?;
    Ok(f64::from_le_bytes(bytes.try_into().expect("8 bytes")))
  }

  /// Reads a UTF-16LE string of `byte_len` bytes.
  pub(crate) fn read_utf16_le(&mut self, byte_len: usize, context: &str) -> DftResult<String> {
    if byte_len % 2 != 0 {
      return Err(DftError::InvalidMetadata {
        context: context.to_string(),
        message: format!("utf16 byte length must be even, got {byte_len}"),
      });
    }
    let bytes = self.read_bytes(byte_len, context)?;
    let units: Vec<u16> = bytes
      .chunks_exact(2)
      .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
      .collect();
    String::from_utf16(&units).map_err(|err| DftError::InvalidMetadata {
      context: context.to_string(),
      message: format!("invalid utf-16: {err}"),
    })
  }
}

/// Validates basic EMF header fields.
pub(crate) fn validate_emf_header(data: &[u8]) -> DftResult<()> {
  if data.len() < 44 {
    return Err(DftError::InvalidEmf {
      sheet_index: 0,
      message: format!("header too short ({} bytes)", data.len()),
    });
  }
  let record_type = u32::from_le_bytes(data[0..4].try_into().expect("4 bytes"));
  if record_type != 1 {
    return Err(DftError::InvalidEmf {
      sheet_index: 0,
      message: format!("expected EMR_HEADER type 1, got {record_type}"),
    });
  }
  let record_size = u32::from_le_bytes(data[4..8].try_into().expect("4 bytes"));
  if record_size < 88 || record_size as usize > data.len() {
    return Err(DftError::InvalidEmf {
      sheet_index: 0,
      message: format!("invalid EMR_HEADER size {record_size}"),
    });
  }
  let signature = u32::from_le_bytes(data[40..44].try_into().expect("4 bytes"));
  if signature != crate::metadata::EMF_SIGNATURE {
    return Err(DftError::InvalidEmf {
      sheet_index: 0,
      message: format!("bad EMF signature 0x{signature:08X}"),
    });
  }
  Ok(())
}
