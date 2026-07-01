//! CLI output formatting.
#![allow(
  clippy::format_collect,
  clippy::format_push_string,
  clippy::missing_errors_doc,
  clippy::redundant_closure_for_method_calls
)]

use std::path::Path;

use dft_reader::InspectReport;
use serde::Serialize;

use crate::OutputFormat;

#[derive(Serialize)]
struct ExtractReport<'a> {
  written: Vec<&'a Path>,
}

/// Renders inspect output to stdout.
pub fn render_inspect(report: &InspectReport, format: OutputFormat) -> anyhow::Result<()> {
  print!("{}", render_inspect_to_string(report, format)?);
  Ok(())
}

/// Renders inspect output as a string (for tests).
pub fn render_inspect_to_string(
  report: &InspectReport,
  format: OutputFormat,
) -> anyhow::Result<String> {
  Ok(match format {
    OutputFormat::Human => {
      let mut out = format!("File: {}\n", report.path.display());
      out.push_str(&format!("Entries: {}\n", report.storage.entries.len()));
      out.push_str(&format!(
        "Viewer info: {} | Document info: {}\n",
        report.storage.has_viewer_info, report.storage.has_document_info
      ));
      if let Some(info) = &report.document_info {
        out.push_str(&format!("Sheets: {}\n", info.number_of_sheets));
      }
      for sheet in &report.sheets {
        out.push_str(&format!(
          "  [{}] {} ({:.1} x {:.1}) emf={} compressed={}\n",
          sheet.index,
          sheet.name,
          sheet.info.width,
          sheet.info.height,
          sheet.info.emf_size,
          sheet.info.emf_compressed_size
        ));
      }
      out
    }
    OutputFormat::Json => serde_json::to_string_pretty(report)?,
  })
}

/// Renders extract output to stdout.
pub fn render_extract(paths: &[impl AsRef<Path>], format: OutputFormat) -> anyhow::Result<()> {
  print!("{}", render_extract_to_string(paths, format)?);
  Ok(())
}

/// Renders extract output as a string (for tests).
pub fn render_extract_to_string(
  paths: &[impl AsRef<Path>],
  format: OutputFormat,
) -> anyhow::Result<String> {
  Ok(match format {
    OutputFormat::Human => paths
      .iter()
      .map(|path| format!("{}\n", path.as_ref().display()))
      .collect(),
    OutputFormat::Json => {
      let refs: Vec<_> = paths.iter().map(|path| path.as_ref()).collect();
      let report = ExtractReport { written: refs };
      serde_json::to_string_pretty(&report)?
    }
  })
}

/// Renders cncKad inspect output to stdout.
pub fn render_cnckad_inspect(
  path: &Path,
  drawing: &drawing_ir::Drawing,
  format: OutputFormat,
) -> anyhow::Result<()> {
  print!(
    "{}",
    render_cnckad_inspect_to_string(path, drawing, format)?
  );
  Ok(())
}

/// Renders cncKad inspect output as a string (for tests).
pub fn render_cnckad_inspect_to_string(
  path: &Path,
  drawing: &drawing_ir::Drawing,
  format: OutputFormat,
) -> anyhow::Result<String> {
  Ok(match format {
    OutputFormat::Human => {
      let mut out = format!("File: {}\nFormat: cncKad text\n", path.display());
      for sheet in &drawing.sheets {
        out.push_str(&format!(
          "Sheet: {} entities={} size={:?}x{:?}\n",
          sheet.name.as_deref().unwrap_or("(unnamed)"),
          sheet.entities.len(),
          sheet.width,
          sheet.height
        ));
      }
      out
    }
    OutputFormat::Json => serde_json::to_string_pretty(drawing)?,
  })
}

/// Renders validate output to stdout.
pub fn render_validate(sheet_count: usize, format: OutputFormat) -> anyhow::Result<()> {
  print!("{}", render_validate_to_string(sheet_count, format));
  Ok(())
}

/// Renders validate output as a string (for tests).
#[must_use]
pub fn render_validate_to_string(sheet_count: usize, format: OutputFormat) -> String {
  match format {
    OutputFormat::Human => format!("validation ok ({sheet_count} sheet(s))\n"),
    OutputFormat::Json => format!("{{\"status\":\"ok\",\"sheets\":{sheet_count}}}"),
  }
}

/// Renders batch conversion summaries to stdout.
pub fn render_convert_all<T: Serialize>(
  summaries: &[T],
  format: OutputFormat,
) -> anyhow::Result<()> {
  print!("{}", render_convert_all_to_string(summaries, format)?);
  Ok(())
}

/// Renders batch conversion summaries as a string (for tests).
pub fn render_convert_all_to_string<T: Serialize>(
  summaries: &[T],
  format: OutputFormat,
) -> anyhow::Result<String> {
  Ok(match format {
    OutputFormat::Human => {
      let mut out = String::new();
      for summary in summaries {
        out.push_str(&serde_json::to_string(summary)?);
        out.push('\n');
      }
      out.push_str(&format!("converted {} file(s)\n", summaries.len()));
      out
    }
    OutputFormat::Json => serde_json::to_string_pretty(summaries)?,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use drawing_ir::Drawing;

  #[test]
  fn validate_json_format() {
    let out = render_validate_to_string(2, OutputFormat::Json);
    assert!(out.contains("\"sheets\":2"));
  }

  #[test]
  fn cnckad_inspect_json_contains_sheets() {
    let drawing = Drawing::new();
    let out =
      render_cnckad_inspect_to_string(Path::new("part.dft"), &drawing, OutputFormat::Json).unwrap();
    assert!(out.contains("\"sheets\""));
  }
}
