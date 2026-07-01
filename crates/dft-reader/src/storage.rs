//! Compound file traversal and metadata parsing.

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use cfb::CompoundFile;

use crate::cfb::{validate_emf_header, ByteCursor};
use crate::decompress::decompress_zlib;
use crate::error::{DftError, DftResult};
use crate::limits::Limits;
use crate::metadata::{
  DraftDocumentInfo, SheetInfo, StorageEntry, StorageEntryKind, StorageTree,
  STORAGE_J_DRAFT_VIEWER_INFO, STREAM_J_DRAFT_DOCUMENT_INFO,
};
use crate::sheet::Sheet;

/// Opens a compound file with file-size validation.
pub(crate) fn open_compound_file(
  path: &Path,
  limits: &Limits,
) -> DftResult<CompoundFile<std::fs::File>> {
  let metadata = std::fs::metadata(path).map_err(|source| DftError::Io {
    path: path.to_path_buf(),
    source,
  })?;
  let file_size = metadata.len();
  if file_size > limits.max_file_size {
    return Err(DftError::limit(
      "max_file_size",
      limits.max_file_size,
      file_size,
    ));
  }
  cfb::open(path).map_err(map_compound_open_error)
}

fn map_compound_open_error(err: std::io::Error) -> DftError {
  if err.kind() == std::io::ErrorKind::InvalidData {
    DftError::NotCompoundFile {
      message: err.to_string(),
    }
  } else {
    DftError::CompoundFile(err)
  }
}

/// Reads an entire stream with size limits.
pub(crate) fn read_stream_limited<R: Read + Seek>(
  compound: &mut CompoundFile<R>,
  path: &str,
  limits: &Limits,
) -> DftResult<Vec<u8>> {
  let mut stream = compound
    .open_stream(path)
    .map_err(|_| DftError::MissingViewerData {
      path: path.to_string(),
    })?;
  let size = stream
    .seek(SeekFrom::End(0))
    .map_err(DftError::CompoundFile)? as u64;
  if size > limits.max_stream_size {
    return Err(DftError::limit(
      "max_stream_size",
      limits.max_stream_size,
      size,
    ));
  }
  stream
    .seek(SeekFrom::Start(0))
    .map_err(DftError::CompoundFile)?;
  let mut data = vec![0u8; size as usize];
  stream
    .read_exact(&mut data)
    .map_err(DftError::CompoundFile)?;
  Ok(data)
}

/// Walks the compound file tree for inspection output.
pub(crate) fn build_storage_tree<R: Read + Seek>(
  compound: &mut CompoundFile<R>,
  limits: &Limits,
) -> DftResult<StorageTree> {
  let mut entries = Vec::new();
  let mut entry_count = 0u32;
  walk_storage(compound, limits, &mut entries, &mut entry_count)?;

  let has_viewer_info = entries.iter().any(|entry| {
    entry.path.ends_with(STORAGE_J_DRAFT_VIEWER_INFO) && entry.kind == StorageEntryKind::Storage
  });
  let document_info_path = format!("/{STORAGE_J_DRAFT_VIEWER_INFO}/{STREAM_J_DRAFT_DOCUMENT_INFO}");
  let has_document_info = entries
    .iter()
    .any(|entry| entry.path == document_info_path && entry.kind == StorageEntryKind::Stream);

  Ok(StorageTree {
    entries,
    has_viewer_info,
    has_document_info,
  })
}

fn walk_storage<R: Read + Seek>(
  compound: &CompoundFile<R>,
  limits: &Limits,
  entries: &mut Vec<StorageEntry>,
  entry_count: &mut u32,
) -> DftResult<()> {
  for entry in compound
    .walk_storage("/")
    .map_err(DftError::CompoundFile)?
  {
    if entry.is_root() {
      continue;
    }

    *entry_count = entry_count.saturating_add(1);
    if *entry_count > limits.max_entry_count {
      return Err(DftError::limit(
        "max_entry_count",
        limits.max_entry_count as u64,
        *entry_count as u64,
      ));
    }

    let entry_path = entry.path().to_string_lossy().replace('\\', "/");
    let entry_depth = entry_path.chars().filter(|ch| *ch == '/').count() as u32;
    if entry_depth > limits.max_storage_depth {
      return Err(DftError::limit(
        "max_storage_depth",
        limits.max_storage_depth as u64,
        entry_depth as u64,
      ));
    }

    if entry.is_storage() {
      entries.push(StorageEntry {
        path: entry_path,
        kind: StorageEntryKind::Storage,
        size: None,
      });
    } else if entry.is_stream() {
      entries.push(StorageEntry {
        path: entry_path,
        kind: StorageEntryKind::Stream,
        size: Some(entry.len()),
      });
    }
  }

  Ok(())
}

/// Parsed draft viewer metadata and sheet list.
#[derive(Debug, Clone)]
pub(crate) struct ParsedDraft {
  /// Document-level metadata.
  pub document_info: DraftDocumentInfo,
  /// Sheet list in document order.
  pub sheets: Vec<Sheet>,
}

/// Parses `JDraftDocumentInfo` and builds sheet metadata.
pub(crate) fn parse_draft_metadata(data: &[u8], limits: &Limits) -> DftResult<ParsedDraft> {
  let mut cursor = ByteCursor::new(data);

  let viewer_info_version = cursor.read_u32_le("JDraftDocumentInfo.viewer_info_version")?;
  let number_of_sheets = cursor.read_i32_le("JDraftDocumentInfo.number_of_sheets")?;
  let active_sheet_index = cursor.read_i32_le("JDraftDocumentInfo.active_sheet_index")?;
  let geometric_version = cursor.read_u32_le("JDraftDocumentInfo.geometric_version")?;
  let units_raw = cursor.read_u16_le("JDraftDocumentInfo.units")?;
  let units = crate::metadata::PaperUnit::from_raw(units_raw);

  if number_of_sheets < 0 {
    return Err(DftError::InvalidMetadata {
      context: "JDraftDocumentInfo.number_of_sheets".to_string(),
      message: format!("negative sheet count {number_of_sheets}"),
    });
  }
  let sheet_count = number_of_sheets as u32;
  if sheet_count > limits.max_sheet_count {
    return Err(DftError::limit(
      "max_sheet_count",
      limits.max_sheet_count as u64,
      sheet_count as u64,
    ));
  }

  let document_info = DraftDocumentInfo {
    viewer_info_version,
    number_of_sheets,
    active_sheet_index,
    geometric_version,
    units,
  };

  let mut sheets = Vec::with_capacity(sheet_count as usize);
  for index in 0..sheet_count {
    let name_units = cursor.read_i32_le("sheet.name_length")?;
    if name_units < 0 {
      return Err(DftError::InvalidMetadata {
        context: format!("sheet[{index}].name_length"),
        message: format!("negative name length {name_units}"),
      });
    }
    let byte_len =
      (name_units as usize)
        .checked_mul(2)
        .ok_or_else(|| DftError::InvalidMetadata {
          context: format!("sheet[{index}].name_length"),
          message: "name length overflow".to_string(),
        })?;
    let mut name = cursor.read_utf16_le(byte_len, &format!("sheet[{index}].name"))?;
    name = name.trim_end_matches('\0').to_string();

    let width = cursor.read_f64_le(&format!("sheet[{index}].width"))?;
    let height = cursor.read_f64_le(&format!("sheet[{index}].height"))?;
    let emf_size = cursor.read_u32_le(&format!("sheet[{index}].emf_size"))?;
    let emf_compressed_size = cursor.read_u32_le(&format!("sheet[{index}].emf_compressed_size"))?;

    sheets.push(Sheet {
      index: index + 1,
      name,
      info: SheetInfo {
        width,
        height,
        emf_size,
        emf_compressed_size,
      },
    });
  }

  Ok(ParsedDraft {
    document_info,
    sheets,
  })
}

/// Parses raw `JDraftDocumentInfo` bytes into document and sheet metadata.
pub fn parse_viewer_document_info(
  data: &[u8],
  limits: &Limits,
) -> DftResult<(DraftDocumentInfo, Vec<Sheet>)> {
  let parsed = parse_draft_metadata(data, limits)?;
  Ok((parsed.document_info, parsed.sheets))
}

/// Extracts and validates one sheet EMF stream.
pub(crate) fn extract_sheet_emf<R: Read + Seek>(
  compound: &mut CompoundFile<R>,
  sheet: &Sheet,
  limits: &Limits,
) -> DftResult<Vec<u8>> {
  let stream_path = format!("{STORAGE_J_DRAFT_VIEWER_INFO}/{}", sheet.index);
  let compressed = read_stream_limited(compound, &stream_path, limits)?;
  let decompressed = decompress_zlib(&compressed, &stream_path, limits)?;

  if sheet.info.emf_size > 0 && decompressed.len() as u32 != sheet.info.emf_size {
    // Warn-level mismatch is tolerated in best-effort mode by caller; here we only
    // validate header and size cap.
  }

  validate_emf_header(&decompressed).map_err(|err| match err {
    DftError::InvalidEmf { message, .. } => DftError::InvalidEmf {
      sheet_index: sheet.index,
      message,
    },
    other => other,
  })?;

  Ok(decompressed)
}
