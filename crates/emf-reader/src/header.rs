//! Structured `EMR_HEADER` parsing and validation.

use crate::error::{EmfError, EmfResult};

/// EMF file signature (`ENHMETA_SIGNATURE`).
pub const EMF_SIGNATURE: u32 = 0x0000_464D;

/// Minimum `EMR_HEADER` record size in bytes.
pub const EMR_HEADER_MIN_SIZE: usize = 88;

/// Parsed fields from an `EMR_HEADER` record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmfHeader {
  /// EMF signature (expected `0x464D`).
  pub signature: u32,
  /// Format version.
  pub version: u32,
  /// Total file size in bytes (`nBytes`).
  pub n_bytes: u32,
  /// Number of records (`nRecords`).
  pub n_records: u32,
  /// GDI handle count.
  pub n_handles: u16,
  /// Logical bounds (`rclBounds`): left, top, right, bottom.
  pub bounds: EmfRectL,
  /// Frame rectangle in 0.01mm units (`rclFrame`).
  pub frame: EmfRectL,
  /// Device size in pixels (`szlDevice`).
  pub device_size: EmfSizeL,
  /// Physical size in 0.01mm (`szlMillimeters`).
  pub millimeter_size: EmfSizeL,
}

/// A Win32 `RECTL` rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmfRectL {
  /// Left edge.
  pub left: i32,
  /// Top edge.
  pub top: i32,
  /// Right edge.
  pub right: i32,
  /// Bottom edge.
  pub bottom: i32,
}

impl EmfRectL {
  /// Returns true when left <= right and top <= bottom.
  #[must_use]
  pub const fn is_valid(&self) -> bool {
    self.left <= self.right && self.top <= self.bottom
  }
}

/// A Win32 `SIZEL` pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmfSizeL {
  /// Width.
  pub width: i32,
  /// Height.
  pub height: i32,
}

impl EmfHeader {
  /// Parses an `EMR_HEADER` record payload (including the 8-byte record header).
  ///
  /// # Errors
  ///
  /// Returns [`EmfError`] when the record is too short, has a bad signature, or invalid bounds.
  pub fn parse(record_data: &[u8]) -> EmfResult<Self> {
    if record_data.len() < EMR_HEADER_MIN_SIZE {
      return Err(EmfError::invalid(
        "header",
        format!("EMR_HEADER too short ({} bytes)", record_data.len()),
      ));
    }

    let signature = read_u32(record_data, 40)?;
    if signature != EMF_SIGNATURE {
      return Err(EmfError::invalid(
        "header",
        format!("invalid EMF signature 0x{signature:08X}"),
      ));
    }

    let bounds = EmfRectL {
      left: read_i32(record_data, 8)?,
      top: read_i32(record_data, 12)?,
      right: read_i32(record_data, 16)?,
      bottom: read_i32(record_data, 20)?,
    };
    if !bounds.is_valid() {
      return Err(EmfError::invalid(
        "header",
        format!(
          "invalid rclBounds ({}, {}, {}, {})",
          bounds.left, bounds.top, bounds.right, bounds.bottom
        ),
      ));
    }

    let frame = EmfRectL {
      left: read_i32(record_data, 24)?,
      top: read_i32(record_data, 28)?,
      right: read_i32(record_data, 32)?,
      bottom: read_i32(record_data, 36)?,
    };

    let header = Self {
      signature,
      version: read_u32(record_data, 44)?,
      n_bytes: read_u32(record_data, 48)?,
      n_records: read_u32(record_data, 52)?,
      n_handles: read_u16(record_data, 56)?,
      bounds,
      frame,
      device_size: EmfSizeL {
        width: read_i32(record_data, 72)?,
        height: read_i32(record_data, 76)?,
      },
      millimeter_size: EmfSizeL {
        width: read_i32(record_data, 80)?,
        height: read_i32(record_data, 84)?,
      },
    };

    Ok(header)
  }

  /// Validates `n_bytes` against the actual buffer length.
  ///
  /// # Errors
  ///
  /// Returns [`EmfError`] when `n_bytes` does not match `actual_len`.
  pub fn validate_n_bytes(&self, actual_len: usize) -> EmfResult<()> {
    let actual = u32::try_from(actual_len).unwrap_or(u32::MAX);
    if self.n_bytes != actual {
      return Err(EmfError::invalid(
        "header",
        format!(
          "nBytes mismatch: header declares {} but buffer is {} bytes",
          self.n_bytes, actual_len
        ),
      ));
    }
    Ok(())
  }

  /// Returns a warning diagnostic code when `n_records` disagrees with the parsed count.
  #[must_use]
  pub fn record_count_mismatch(&self, parsed_count: u32) -> Option<String> {
    if self.n_records == parsed_count {
      None
    } else {
      Some(format!(
        "header nRecords={} but parser found {parsed_count} records",
        self.n_records
      ))
    }
  }
}

fn read_u32(data: &[u8], offset: usize) -> EmfResult<u32> {
  let bytes: [u8; 4] = data
    .get(offset..offset + 4)
    .ok_or_else(|| EmfError::invalid("header", "truncated u32 field"))?
    .try_into()
    .map_err(|_| EmfError::invalid("header", "truncated u32 field"))?;
  Ok(u32::from_le_bytes(bytes))
}

fn read_u16(data: &[u8], offset: usize) -> EmfResult<u16> {
  let bytes: [u8; 2] = data
    .get(offset..offset + 2)
    .ok_or_else(|| EmfError::invalid("header", "truncated u16 field"))?
    .try_into()
    .map_err(|_| EmfError::invalid("header", "truncated u16 field"))?;
  Ok(u16::from_le_bytes(bytes))
}

fn read_i32(data: &[u8], offset: usize) -> EmfResult<i32> {
  let bytes: [u8; 4] = data
    .get(offset..offset + 4)
    .ok_or_else(|| EmfError::invalid("header", "truncated i32 field"))?
    .try_into()
    .map_err(|_| EmfError::invalid("header", "truncated i32 field"))?;
  Ok(i32::from_le_bytes(bytes))
}
