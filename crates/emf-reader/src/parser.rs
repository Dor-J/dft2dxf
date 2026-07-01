//! Strict EMF record iterator.

use crate::error::{EmfError, EmfResult};
use crate::record::{EMR_EOF, EMR_HEADER};

/// Maximum records to parse from one EMF by default.
pub const DEFAULT_MAX_RECORD_COUNT: u32 = 1_000_000;

/// Maximum individual record size.
pub const DEFAULT_MAX_RECORD_SIZE: u32 = 16 * 1024 * 1024;

/// Classification for a parsed record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordClass {
  /// Header/control record.
  Control,
  /// State record.
  State,
  /// Drawing record with geometry.
  Drawing,
  /// Unsupported but recognized record.
  Unsupported,
  /// Terminal EOF record.
  Eof,
}

/// One EMF record with raw payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmfRecord {
  /// Zero-based record index.
  pub index: u32,
  /// Record type field.
  pub record_type: u32,
  /// Total record size in bytes.
  pub size: u32,
  /// Raw record bytes including header.
  pub data: Vec<u8>,
}

impl EmfRecord {
  /// Returns record classification.
  #[must_use]
  pub fn class(&self) -> RecordClass {
    match self.record_type {
      EMR_EOF => RecordClass::Eof,
      EMR_HEADER | 17 | 35 | 36 | 37 | 38 | 82 | 95 => RecordClass::Control,
      4 | 5 | 27 | 42 | 45 | 46 | 47 | 49 | 54 | 55 | 83 | 84 | 86 | 87 | 88 => {
        RecordClass::Drawing
      }
      _ => RecordClass::Unsupported,
    }
  }
}

/// Parsed EMF with record list.
#[derive(Debug, Clone)]
pub struct EmfDocument {
  /// Parsed records excluding padding.
  pub records: Vec<EmfRecord>,
}

impl EmfDocument {
  /// Parses EMF bytes into records with limits.
  ///
  /// # Errors
  ///
  /// Returns [`EmfError`] if the EMF is malformed, omits EOF, or exceeds configured limits.
  pub fn parse(data: &[u8], max_records: u32, max_record_size: u32) -> EmfResult<Self> {
    if data.len() < 8 {
      return Err(EmfError::invalid("header", "EMF too short"));
    }

    let mut offset = 0usize;
    let mut records = Vec::new();
    let mut index = 0u32;
    let mut saw_eof = false;

    while offset < data.len() {
      if index >= max_records {
        return Err(EmfError::limit(
          "max_record_count",
          u64::from(max_records),
          u64::from(index),
        ));
      }

      if data.len() - offset < 8 {
        return Err(EmfError::invalid(
          format!("record[{index}]"),
          "truncated record header",
        ));
      }

      let record_type = read_u32_le(data, offset);
      let size = read_u32_le(data, offset + 4);

      if size < 8 || size % 4 != 0 {
        return Err(EmfError::invalid(
          format!("record[{index}]"),
          format!("invalid record size {size}"),
        ));
      }
      if size > max_record_size {
        return Err(EmfError::limit(
          "max_record_size",
          u64::from(max_record_size),
          u64::from(size),
        ));
      }

      let end = offset
        .checked_add(size as usize)
        .ok_or_else(|| EmfError::invalid(format!("record[{index}]"), "record size overflow"))?;
      if end > data.len() {
        return Err(EmfError::invalid(
          format!("record[{index}]"),
          "record extends past EMF end",
        ));
      }

      let record_data = data[offset..end].to_vec();
      let record = EmfRecord {
        index,
        record_type,
        size,
        data: record_data,
      };

      if record.record_type == EMR_EOF {
        saw_eof = true;
        records.push(record);
        if end != data.len() {
          // Trailing bytes after EOF are ignored per defensive parsing policy.
        }
        break;
      }

      records.push(record);
      offset = end;
      index += 1;
    }

    if !saw_eof {
      return Err(EmfError::MissingEof);
    }

    Ok(Self { records })
  }
}

fn read_u32_le(data: &[u8], offset: usize) -> u32 {
  u32::from_le_bytes([
    data[offset],
    data[offset + 1],
    data[offset + 2],
    data[offset + 3],
  ])
}
