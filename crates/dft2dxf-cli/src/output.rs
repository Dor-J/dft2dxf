//! CLI output formatting.

use std::path::Path;

use dft_reader::InspectReport;
use serde::Serialize;

use crate::OutputFormat;

#[derive(Serialize)]
struct ExtractReport<'a> {
  written: Vec<&'a Path>,
}

/// Renders inspect output.
pub fn render_inspect(report: &InspectReport, format: OutputFormat) -> anyhow::Result<()> {
  match format {
    OutputFormat::Human => {
      println!("File: {}", report.path.display());
      println!("Entries: {}", report.storage.entries.len());
      println!(
        "Viewer info: {} | Document info: {}",
        report.storage.has_viewer_info, report.storage.has_document_info
      );
      if let Some(info) = &report.document_info {
        println!("Sheets: {}", info.number_of_sheets);
      }
      for sheet in &report.sheets {
        println!(
          "  [{}] {} ({:.1} x {:.1}) emf={} compressed={}",
          sheet.index,
          sheet.name,
          sheet.info.width,
          sheet.info.height,
          sheet.info.emf_size,
          sheet.info.emf_compressed_size
        );
      }
    }
    OutputFormat::Json => {
      println!("{}", serde_json::to_string_pretty(report)?);
    }
  }
  Ok(())
}

/// Renders extract output.
pub fn render_extract(paths: &[impl AsRef<Path>], format: OutputFormat) -> anyhow::Result<()> {
  match format {
    OutputFormat::Human => {
      for path in paths {
        println!("{}", path.as_ref().display());
      }
    }
    OutputFormat::Json => {
      let refs: Vec<_> = paths.iter().map(|path| path.as_ref()).collect();
      let report = ExtractReport { written: refs };
      println!("{}", serde_json::to_string_pretty(&report)?);
    }
  }
  Ok(())
}

/// Renders validate output.
pub fn render_validate(sheet_count: usize, format: OutputFormat) -> anyhow::Result<()> {
  match format {
    OutputFormat::Human => {
      println!("validation ok ({sheet_count} sheet(s))");
    }
    OutputFormat::Json => {
      println!("{{\"status\":\"ok\",\"sheets\":{sheet_count}}}");
    }
  }
  Ok(())
}
