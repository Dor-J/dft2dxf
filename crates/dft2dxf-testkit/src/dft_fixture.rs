//! Build minimal synthetic `.dft` compound files for tests.

use std::io::Write;
use std::path::Path;

use cfb::CompoundFile;
use flate2::write::ZlibEncoder;
use flate2::Compression;

/// One sheet in a synthetic DFT.
#[derive(Debug, Clone)]
pub struct SheetSpec {
  /// Sheet display name.
  pub name: String,
  /// Sheet width.
  pub width: f64,
  /// Sheet height.
  pub height: f64,
  /// EMF payload to embed (uncompressed).
  pub emf: Vec<u8>,
  /// Optional compressed stream override for negative tests.
  pub compressed_override: Option<Vec<u8>>,
}

/// Specification for a minimal synthetic DFT.
#[derive(Debug, Clone)]
pub struct MinimalDftSpec {
  /// Sheets to include.
  pub sheets: Vec<SheetSpec>,
}

impl MinimalDftSpec {
  /// Creates a one-sheet synthetic DFT spec.
  #[must_use]
  pub fn one_sheet(name: impl Into<String>, emf: Vec<u8>) -> Self {
    Self {
      sheets: vec![SheetSpec {
        name: name.into(),
        width: 297.0,
        height: 210.0,
        emf,
        compressed_override: None,
      }],
    }
  }

  /// Creates a multi-sheet synthetic DFT spec.
  #[must_use]
  pub fn multi_sheet(sheets: Vec<(String, Vec<u8>)>) -> Self {
    Self {
      sheets: sheets
        .into_iter()
        .map(|(name, emf)| SheetSpec {
          name,
          width: 297.0,
          height: 210.0,
          emf,
          compressed_override: None,
        })
        .collect(),
    }
  }
}

/// Writes a synthetic `.dft` compound file to `path`.
///
/// # Errors
///
/// Returns an I/O error if the compound file, storage, metadata stream, or sheet streams cannot
/// be created or written.
pub fn build_minimal_dft(path: &Path, spec: &MinimalDftSpec) -> std::io::Result<()> {
  if let Some(parent) = path.parent() {
    if !parent.as_os_str().is_empty() {
      std::fs::create_dir_all(parent)?;
    }
  }

  let file = std::fs::File::create(path)?;
  let mut compound = CompoundFile::create(file)?;

  compound.create_storage_all("JDraftViewerInfo")?;
  write_document_info(&mut compound, spec)?;
  for (index, sheet) in spec.sheets.iter().enumerate() {
    let stream_name = (index + 1).to_string();
    write_sheet_stream(&mut compound, &stream_name, sheet)?;
  }

  compound.flush()?;
  Ok(())
}

fn write_document_info(
  compound: &mut CompoundFile<std::fs::File>,
  spec: &MinimalDftSpec,
) -> std::io::Result<()> {
  let mut data = Vec::new();
  data.extend_from_slice(&1u32.to_le_bytes()); // viewer_info_version
  data.extend_from_slice(&usize_to_i32(spec.sheets.len(), "sheet count")?.to_le_bytes());
  data.extend_from_slice(&0i32.to_le_bytes()); // active_sheet_index
  data.extend_from_slice(&1u32.to_le_bytes()); // geometric_version
  data.extend_from_slice(&0x003Du16.to_le_bytes()); // millimeters

  for sheet in &spec.sheets {
    let name_utf16: Vec<u16> = sheet.name.encode_utf16().collect();
    data.extend_from_slice(&usize_to_i32(name_utf16.len(), "sheet name length")?.to_le_bytes());
    for unit in name_utf16 {
      data.extend_from_slice(&unit.to_le_bytes());
    }
    data.extend_from_slice(&sheet.width.to_le_bytes());
    data.extend_from_slice(&sheet.height.to_le_bytes());
    let emf_size = usize_to_u32(sheet.emf.len(), "EMF size")?;
    let compressed = if let Some(payload) = &sheet.compressed_override {
      payload.clone()
    } else {
      compress_zlib(&sheet.emf)?
    };
    data.extend_from_slice(&emf_size.to_le_bytes());
    data.extend_from_slice(&usize_to_u32(compressed.len(), "compressed EMF size")?.to_le_bytes());
  }

  let mut stream = compound.create_stream("JDraftViewerInfo/JDraftDocumentInfo")?;
  stream.write_all(&data)?;
  Ok(())
}

fn write_sheet_stream(
  compound: &mut CompoundFile<std::fs::File>,
  stream_name: &str,
  sheet: &SheetSpec,
) -> std::io::Result<()> {
  let compressed = if let Some(payload) = &sheet.compressed_override {
    payload.clone()
  } else {
    compress_zlib(&sheet.emf)?
  };
  let path = format!("JDraftViewerInfo/{stream_name}");
  let mut stream = compound.create_stream(&path)?;
  stream.write_all(&compressed)?;
  Ok(())
}

fn compress_zlib(data: &[u8]) -> std::io::Result<Vec<u8>> {
  let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
  encoder.write_all(data)?;
  encoder.finish()
}

fn usize_to_i32(value: usize, context: &str) -> std::io::Result<i32> {
  i32::try_from(value).map_err(|_| {
    std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!("{context} does not fit in i32"),
    )
  })
}

fn usize_to_u32(value: usize, context: &str) -> std::io::Result<u32> {
  u32::try_from(value).map_err(|_| {
    std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!("{context} does not fit in u32"),
    )
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::emf_fixture;
  use tempfile::tempdir;

  #[test]
  fn synthetic_dft_is_emf_extractable() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sample.dft");
    let emf = emf_fixture::build_rectangle_emf(0, 0, 100, 50);
    build_minimal_dft(&path, &MinimalDftSpec::one_sheet("Sheet1", emf)).unwrap();
    assert!(path.exists());
  }
}
