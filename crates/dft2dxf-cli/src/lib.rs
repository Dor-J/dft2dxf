//! `dft2dxf` library — CLI logic and helpers.

#![forbid(unsafe_code)]
#![allow(
  clippy::items_after_statements,
  clippy::match_same_arms,
  clippy::missing_errors_doc,
  clippy::ptr_arg,
  clippy::too_many_arguments
)]

pub mod output;

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ckad_reader::{detect_format, read_to_drawing, DftContainerFormat};
use clap::{Parser, Subcommand, ValueEnum};
use dft_reader::{DftDocument, DftOpenOptions, Limits};
use drawing_ir::PaperUnit;
use tracing_subscriber::EnvFilter;

/// CLI root arguments.
#[derive(Debug, Parser)]
#[command(
  name = "dft2dxf",
  version,
  about = "Convert .dft draft files (Solid Edge or cncKad) to portable vector outputs",
  long_about = "Reads Solid Edge compound .dft files via embedded EMF viewer data, or \
                Metalix cncKad text .dft geometry, and converts visible vector representation \
                to DXF/SVG."
)]
pub struct Cli {
  /// Command to execute.
  #[command(subcommand)]
  pub command: Option<Commands>,

  /// Input `.dft` file (shorthand for convert when used with --output).
  #[arg(value_name = "INPUT")]
  pub input: Option<PathBuf>,

  /// Output `.dxf` file (experimental convert shorthand).
  #[arg(short, long, value_name = "FILE")]
  pub output: Option<PathBuf>,

  /// One-based sheet index.
  #[arg(long, value_name = "INDEX")]
  pub sheet: Option<u32>,

  /// Output format for structured diagnostics.
  #[arg(long, value_enum, default_value_t = OutputFormat::Human, global = true)]
  pub format: OutputFormat,

  /// Override maximum input file size in bytes.
  #[arg(long)]
  pub max_file_size: Option<u64>,

  /// Override maximum compressed stream size in bytes.
  #[arg(long)]
  pub max_stream_size: Option<u64>,

  /// Override maximum decompressed EMF size in bytes.
  #[arg(long)]
  pub max_decompressed_size: Option<u64>,

  /// Use gitignored local `.dft` fixtures under `tests/fixtures/valid/local/`.
  #[arg(short = 'l', long = "local", global = true)]
  pub local: bool,

  /// Write CAM/metadata JSON sidecar next to DXF output.
  #[arg(long = "cam-json", global = true)]
  pub cam_json: bool,

  /// Override output drawing units (`mm`, `in`, `unitless`).
  #[arg(long = "units", value_name = "UNIT", global = true)]
  pub units: Option<String>,
}

/// CLI subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
  /// Inspect compound file structure and draft metadata.
  Inspect {
    /// Input `.dft` file.
    input: PathBuf,
  },
  /// Extract embedded EMF sheet streams.
  ExtractEmf {
    /// Input `.dft` file.
    input: PathBuf,
    /// Output directory for `.emf` files.
    #[arg(long, value_name = "DIR")]
    output_dir: PathBuf,
    /// One-based sheet index (default: all sheets).
    #[arg(long)]
    sheet: Option<u32>,
  },
  /// Validate `.dft` structure without writing conversion output.
  Validate {
    /// Input `.dft` file.
    input: PathBuf,
  },
  /// Validate all `.dft` fixtures under the repository valid-fixtures directory.
  ValidateFixtures,
  /// Convert extracted visible representation to DXF (experimental).
  Convert {
    /// Input `.dft` file.
    input: PathBuf,
    /// Output `.dxf` file.
    #[arg(short, long)]
    output: PathBuf,
    /// One-based sheet index.
    #[arg(long)]
    sheet: Option<u32>,
    /// Also write SVG preview to directory.
    #[arg(long, value_name = "DIR")]
    svg_preview: Option<PathBuf>,
  },
  /// Batch-convert fixtures or a directory of `.dft` files to DXF/SVG.
  ConvertAll {
    /// Output directory for `.dxf` files.
    #[arg(long, value_name = "DIR")]
    dxf_dir: PathBuf,
    /// Output directory for per-file SVG folders.
    #[arg(long, value_name = "DIR")]
    svg_dir: PathBuf,
    /// Optional input directory (defaults to active valid fixtures dir).
    #[arg(long, value_name = "DIR")]
    input_dir: Option<PathBuf>,
  },
}

/// Structured CLI output format.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
  /// Human-readable text output.
  Human,
  /// Machine-readable JSON output.
  Json,
}

/// Batch conversion summary for one input file.
#[derive(Debug, Default, serde::Serialize)]
pub struct ConvertSummary {
  /// Input path.
  pub input: String,
  /// Whether conversion succeeded.
  pub ok: bool,
  /// Total entity count.
  pub entities: usize,
  /// Distinct layer count.
  pub layers: usize,
  /// Whether CAM data was present.
  pub has_cam: bool,
  /// Diagnostic count.
  pub diagnostics: usize,
  /// Error message when `ok` is false.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error: Option<String>,
}

/// Initializes tracing from `RUST_LOG`.
pub fn init_tracing() {
  let _ = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .try_init();
}

/// Runs the CLI from parsed arguments.
pub fn run(cli: Cli) -> Result<()> {
  let limits = build_limits(&cli);

  match cli.command {
    Some(Commands::Inspect { input }) => cmd_inspect(&input, limits, cli.format),
    Some(Commands::ExtractEmf {
      input,
      output_dir,
      sheet,
    }) => cmd_extract_emf(&input, &output_dir, sheet, limits, cli.format),
    Some(Commands::Validate { input }) => cmd_validate(&input, limits, cli.format),
    Some(Commands::ValidateFixtures) => cmd_validate_fixtures(limits, cli.local, cli.format),
    Some(Commands::Convert {
      input,
      output,
      sheet,
      svg_preview,
    }) => cmd_convert(
      &input,
      &output,
      sheet,
      svg_preview,
      limits,
      cli.cam_json,
      cli.units.as_deref(),
    ),
    Some(Commands::ConvertAll {
      dxf_dir,
      svg_dir,
      input_dir,
    }) => cmd_convert_all(
      &dxf_dir,
      &svg_dir,
      input_dir.as_deref(),
      limits,
      cli.local,
      cli.cam_json,
      cli.units.as_deref(),
      cli.format,
    ),
    None => {
      let input = cli
        .input
        .context("missing input .dft file; pass a path or use a subcommand")?;
      let output = cli
        .output
        .context("missing --output; use `dft2dxf convert` for explicit conversion")?;
      cmd_convert(
        &input,
        &output,
        cli.sheet,
        None,
        limits,
        cli.cam_json,
        cli.units.as_deref(),
      )
    }
  }
}

/// Builds resource limits from CLI overrides.
#[must_use]
pub fn build_limits(cli: &Cli) -> Limits {
  let mut limits = Limits::strict();
  if let Some(value) = cli.max_file_size {
    limits.max_file_size = value;
  }
  if let Some(value) = cli.max_stream_size {
    limits.max_stream_size = value;
  }
  if let Some(value) = cli.max_decompressed_size {
    limits.max_decompressed_size = value;
  }
  limits
}

fn cmd_inspect(input: &PathBuf, limits: Limits, format: OutputFormat) -> Result<()> {
  match sniff_format(input)? {
    DftContainerFormat::CncKad => cmd_inspect_cnckad(input, limits, format),
    DftContainerFormat::SolidEdgeCompound => {
      let mut document =
        DftDocument::open_with_options(input, &DftOpenOptions::new().with_limits(limits))?;
      let report = document.inspect().context("inspect failed")?;
      output::render_inspect(&report, format)?;
      Ok(())
    }
  }
}

fn cmd_extract_emf(
  input: &PathBuf,
  output_dir: &PathBuf,
  sheet: Option<u32>,
  limits: Limits,
  format: OutputFormat,
) -> Result<()> {
  let mut document =
    DftDocument::open_with_options(input, &DftOpenOptions::new().with_limits(limits))?;
  let sheets = document.sheets().context("failed to read sheets")?;
  let targets: Vec<u32> = match sheet {
    Some(index) => vec![index],
    None => sheets.iter().map(|value| value.index).collect(),
  };

  let mut written = Vec::new();
  for index in targets {
    let extracted = document
      .extract_emf(index)
      .with_context(|| format!("failed to extract sheet {index}"))?;
    let file_name = format!("sheet-{index}.emf");
    let path = output_dir.join(file_name);
    extracted
      .write_to(&path)
      .with_context(|| format!("failed to write {}", path.display()))?;
    written.push(path);
  }

  output::render_extract(&written, format)?;
  Ok(())
}

fn cmd_validate(input: &PathBuf, limits: Limits, format: OutputFormat) -> Result<()> {
  match sniff_format(input)? {
    DftContainerFormat::CncKad => cmd_validate_cnckad(input, limits, format),
    DftContainerFormat::SolidEdgeCompound => {
      let mut document =
        DftDocument::open_with_options(input, &DftOpenOptions::new().with_limits(limits))?;
      let report = document.inspect().context("inspect failed")?;
      if !report.storage.has_viewer_info || !report.storage.has_document_info {
        anyhow::bail!("missing required JDraftViewerInfo metadata");
      }
      let sheets = document.sheets().context("failed to read sheets")?;
      for sheet in &sheets {
        document
          .extract_emf(sheet.index)
          .with_context(|| format!("sheet {} EMF validation failed", sheet.index))?;
      }
      output::render_validate(sheets.len(), format)?;
      Ok(())
    }
  }
}

fn cmd_convert(
  input: &PathBuf,
  output: &PathBuf,
  sheet: Option<u32>,
  svg_preview: Option<PathBuf>,
  limits: Limits,
  cam_json: bool,
  units: Option<&str>,
) -> Result<()> {
  match sniff_format(input)? {
    DftContainerFormat::CncKad => {
      if sheet.is_some() {
        tracing::warn!("cncKad .dft files contain a single geometry sheet; --sheet is ignored");
      }
      cmd_convert_cnckad(input, output, svg_preview, limits, cam_json, units)
    }
    DftContainerFormat::SolidEdgeCompound => {
      cmd_convert_solid_edge(input, output, sheet, svg_preview, limits, cam_json, units)
    }
  }
}

fn cmd_convert_solid_edge(
  input: &PathBuf,
  output: &PathBuf,
  sheet: Option<u32>,
  svg_preview: Option<PathBuf>,
  limits: Limits,
  cam_json: bool,
  units: Option<&str>,
) -> Result<()> {
  let mut document =
    DftDocument::open_with_options(input, &DftOpenOptions::new().with_limits(limits))?;
  let sheets = document.sheets().context("failed to read sheets")?;
  let index = sheet.unwrap_or_else(|| sheets.first().map_or(1, |s| s.index));
  let sheet_meta = document.sheet(index).context("sheet lookup failed")?;
  let emf = document
    .extract_emf(index)
    .with_context(|| format!("failed to extract sheet {index}"))?;

  let emf_doc = emf_reader::EmfDocument::parse(
    &emf.data,
    emf_reader::DEFAULT_MAX_RECORD_COUNT,
    emf_reader::DEFAULT_MAX_RECORD_SIZE,
  )
  .context("EMF parse failed")?;

  let mut drawing = emf_reader::replay_to_drawing(
    &emf_doc,
    Some(sheet_meta.index),
    Some(sheet_meta.name.clone()),
    Some(sheet_meta.info.width),
    Some(sheet_meta.info.height),
  );

  apply_units_override(&mut drawing, units);

  drawing_dxf::write_drawing_to_file(&mut drawing, output)
    .with_context(|| format!("failed to write DXF to {}", output.display()))?;

  if cam_json {
    write_cam_json_sidecar(&drawing, output)?;
  }

  if let Some(dir) = svg_preview {
    let svg_path = dir.join(format!("sheet-{index}.svg"));
    drawing_svg::write_drawing_to_file(&drawing, &svg_path)
      .with_context(|| format!("failed to write SVG to {}", svg_path.display()))?;
  }

  tracing::info!(output = %output.display(), sheet = index, "conversion complete");
  Ok(())
}

fn cmd_convert_cnckad(
  input: &PathBuf,
  output: &PathBuf,
  svg_preview: Option<PathBuf>,
  limits: Limits,
  cam_json: bool,
  units: Option<&str>,
) -> Result<()> {
  let mut drawing = read_to_drawing(input, limits.max_file_size)
    .with_context(|| format!("failed to read cncKad file {}", input.display()))?;

  apply_units_override(&mut drawing, units);

  drawing_dxf::write_drawing_to_file(&mut drawing, output)
    .with_context(|| format!("failed to write DXF to {}", output.display()))?;

  if cam_json {
    write_cam_json_sidecar(&drawing, output)?;
  }

  if let Some(dir) = svg_preview {
    std::fs::create_dir_all(&dir)?;
    let svg_path = dir.join("sheet-1.svg");
    drawing_svg::write_drawing_to_file(&drawing, &svg_path)
      .with_context(|| format!("failed to write SVG to {}", svg_path.display()))?;
  }

  tracing::info!(output = %output.display(), format = "cnckad", "conversion complete");
  Ok(())
}

fn cmd_convert_all(
  dxf_dir: &PathBuf,
  svg_dir: &PathBuf,
  input_dir: Option<&Path>,
  limits: Limits,
  local: bool,
  cam_json: bool,
  units: Option<&str>,
  format: OutputFormat,
) -> Result<()> {
  std::fs::create_dir_all(dxf_dir)?;
  std::fs::create_dir_all(svg_dir)?;

  let fixtures = if let Some(dir) = input_dir {
    discover_dft_files(dir)?
  } else {
    dft2dxf_testkit::discover_valid_dft_fixtures(local)
  };

  if fixtures.is_empty() {
    anyhow::bail!("no .dft files found for batch conversion");
  }

  let mut summaries = Vec::new();
  for input in &fixtures {
    let stem = input
      .file_stem()
      .and_then(|value| value.to_str())
      .unwrap_or("output");
    let dxf_path = dxf_dir.join(format!("{stem}.dxf"));
    let svg_folder = svg_dir.join(stem);
    std::fs::create_dir_all(&svg_folder)?;

    let result = (|| -> Result<ConvertSummary> {
      cmd_convert(
        input,
        &dxf_path,
        None,
        Some(svg_folder.clone()),
        limits,
        cam_json,
        units,
      )?;
      let drawing = load_drawing_for_summary(input, limits)?;
      Ok(summarize_drawing(input, &drawing))
    })();

    summaries.push(match result {
      Ok(summary) => summary,
      Err(err) => ConvertSummary {
        input: input.display().to_string(),
        ok: false,
        error: Some(err.to_string()),
        ..ConvertSummary::default()
      },
    });
  }

  output::render_convert_all(&summaries, format)?;
  if summaries.iter().any(|summary| !summary.ok) {
    anyhow::bail!("one or more batch conversions failed");
  }
  Ok(())
}

/// Lists `.dft` files in a directory (non-recursive).
pub fn discover_dft_files(dir: &Path) -> Result<Vec<PathBuf>> {
  let mut files = Vec::new();
  for entry in std::fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    if path
      .extension()
      .and_then(|ext| ext.to_str())
      .is_some_and(|ext| ext.eq_ignore_ascii_case("dft"))
    {
      files.push(path);
    }
  }
  files.sort();
  Ok(files)
}

fn load_drawing_for_summary(input: &Path, limits: Limits) -> Result<drawing_ir::Drawing> {
  match sniff_format(input)? {
    DftContainerFormat::CncKad => {
      read_to_drawing(input, limits.max_file_size).map_err(|err| anyhow::anyhow!(err.to_string()))
    }
    DftContainerFormat::SolidEdgeCompound => {
      let mut document =
        DftDocument::open_with_options(input, &DftOpenOptions::new().with_limits(limits))?;
      let index = document.sheets()?.first().map_or(1, |sheet| sheet.index);
      let sheet_meta = document.sheet(index)?;
      let emf = document.extract_emf(index)?;
      let emf_doc = emf_reader::EmfDocument::parse(
        &emf.data,
        emf_reader::DEFAULT_MAX_RECORD_COUNT,
        emf_reader::DEFAULT_MAX_RECORD_SIZE,
      )?;
      Ok(emf_reader::replay_to_drawing(
        &emf_doc,
        Some(sheet_meta.index),
        Some(sheet_meta.name.clone()),
        Some(sheet_meta.info.width),
        Some(sheet_meta.info.height),
      ))
    }
  }
}

/// Builds a conversion summary from a loaded drawing.
#[must_use]
pub fn summarize_drawing(input: &Path, drawing: &drawing_ir::Drawing) -> ConvertSummary {
  let entities: usize = drawing
    .sheets
    .iter()
    .map(|sheet| sheet.entities.len())
    .sum();
  let layers: usize = drawing
    .sheets
    .iter()
    .flat_map(|sheet| sheet.entities.iter())
    .filter_map(|entity| entity.layer.as_ref())
    .collect::<std::collections::BTreeSet<_>>()
    .len();
  ConvertSummary {
    input: input.display().to_string(),
    ok: true,
    entities,
    layers,
    has_cam: drawing.cam.is_some(),
    diagnostics: drawing.diagnostics.len(),
    error: None,
  }
}

/// Applies a CLI units override to drawing metadata.
pub fn apply_units_override(drawing: &mut drawing_ir::Drawing, units: Option<&str>) {
  let Some(units) = units else {
    return;
  };
  drawing.metadata.units = match units.to_ascii_lowercase().as_str() {
    "mm" | "millimeters" => PaperUnit::Millimeters,
    "in" | "inches" => PaperUnit::Inches,
    "unitless" => PaperUnit::Unitless,
    _ => PaperUnit::Millimeters,
  };
}

/// Writes CAM/metadata JSON next to a DXF output path.
pub fn write_cam_json_sidecar(drawing: &drawing_ir::Drawing, dxf_output: &Path) -> Result<()> {
  let sidecar = dxf_output.with_extension("cam.json");
  #[derive(serde::Serialize)]
  struct Sidecar<'a> {
    metadata: &'a drawing_ir::DrawingMetadata,
    cam: &'a Option<drawing_ir::CamProgram>,
    diagnostics: &'a [drawing_ir::Diagnostic],
  }
  let payload = Sidecar {
    metadata: &drawing.metadata,
    cam: &drawing.cam,
    diagnostics: &drawing.diagnostics,
  };
  std::fs::write(&sidecar, serde_json::to_string_pretty(&payload)?)?;
  Ok(())
}

fn cmd_inspect_cnckad(input: &PathBuf, limits: Limits, format: OutputFormat) -> Result<()> {
  let drawing = read_to_drawing(input, limits.max_file_size)
    .with_context(|| format!("failed to read cncKad file {}", input.display()))?;
  output::render_cnckad_inspect(input, &drawing, format)?;
  Ok(())
}

fn cmd_validate_cnckad(input: &PathBuf, limits: Limits, format: OutputFormat) -> Result<()> {
  let drawing = read_to_drawing(input, limits.max_file_size)
    .with_context(|| format!("failed to read cncKad file {}", input.display()))?;
  let entity_count: usize = drawing
    .sheets
    .iter()
    .map(|sheet| sheet.entities.len())
    .sum();
  if entity_count == 0 {
    anyhow::bail!("no geometry entities found in cncKad file");
  }
  output::render_validate(drawing.sheets.len(), format)?;
  Ok(())
}

/// Detects `.dft` container format from the file header.
pub fn sniff_format(path: &Path) -> Result<DftContainerFormat> {
  let mut file =
    std::fs::File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
  let mut header = [0u8; 12];
  file
    .read_exact(&mut header)
    .with_context(|| format!("failed to read header from {}", path.display()))?;
  detect_format(&header).ok_or_else(|| {
    anyhow::anyhow!(
      "unrecognized .dft format in {} (expected Solid Edge compound file or cncKad text)",
      path.display()
    )
  })
}

fn cmd_validate_fixtures(limits: Limits, local: bool, format: OutputFormat) -> Result<()> {
  let fixtures = dft2dxf_testkit::discover_valid_dft_fixtures(local);
  if fixtures.is_empty() {
    let dir = dft2dxf_testkit::active_valid_fixtures_dir(local);
    anyhow::bail!(
      "no .dft fixtures found in {} (use --local for tests/fixtures/valid/local/)",
      dir.display()
    );
  }

  let mut validated = 0usize;
  for path in &fixtures {
    cmd_validate(path, limits, format)
      .with_context(|| format!("fixture validation failed for {}", path.display()))?;
    validated += 1;
  }

  match format {
    OutputFormat::Human => {
      println!(
        "validated {validated} fixture(s) in {}",
        dft2dxf_testkit::active_valid_fixtures_dir(local).display()
      );
    }
    OutputFormat::Json => {
      println!("{{\"status\":\"ok\",\"fixtures\":{validated},\"local\":{local}}}");
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use dft2dxf_testkit::{minimal_cnckad_dft, professional_cnckad_dft};
  use drawing_ir::Drawing;

  #[test]
  fn apply_units_override_sets_millimeters() {
    let mut drawing = Drawing::new();
    apply_units_override(&mut drawing, Some("mm"));
    assert_eq!(drawing.metadata.units, PaperUnit::Millimeters);
  }

  #[test]
  fn apply_units_override_sets_inches() {
    let mut drawing = Drawing::new();
    apply_units_override(&mut drawing, Some("inches"));
    assert_eq!(drawing.metadata.units, PaperUnit::Inches);
  }

  #[test]
  fn summarize_drawing_counts_entities_and_layers() {
    use ckad_reader::parse_content;
    let drawing = parse_content(&professional_cnckad_dft(), None).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.dft");
    std::fs::write(&path, professional_cnckad_dft()).unwrap();
    let summary = summarize_drawing(&path, &drawing);
    assert!(summary.ok);
    assert!(summary.entities > 0);
    assert!(summary.has_cam);
  }

  #[test]
  fn discover_dft_files_finds_dft_extension() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.dft"), b"x").unwrap();
    std::fs::write(dir.path().join("b.txt"), b"x").unwrap();
    let files = discover_dft_files(dir.path()).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].to_string_lossy().ends_with("a.dft"));
  }

  #[test]
  fn write_cam_json_sidecar_creates_file() {
    use ckad_reader::parse_content;
    let drawing = parse_content(&professional_cnckad_dft(), None).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let dxf = dir.path().join("out.dxf");
    std::fs::write(&dxf, b"").unwrap();
    write_cam_json_sidecar(&drawing, &dxf).unwrap();
    let sidecar = dir.path().join("out.cam.json");
    assert!(sidecar.exists());
    let content = std::fs::read_to_string(sidecar).unwrap();
    assert!(content.contains("\"cam\""));
  }

  #[test]
  fn sniff_format_detects_cnckad() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("part.dft");
    std::fs::write(&path, minimal_cnckad_dft()).unwrap();
    assert_eq!(sniff_format(&path).unwrap(), DftContainerFormat::CncKad);
  }

  #[test]
  fn build_limits_respects_overrides() {
    let cli = Cli {
      command: None,
      input: None,
      output: None,
      sheet: None,
      format: OutputFormat::Human,
      max_file_size: Some(999),
      max_stream_size: Some(888),
      max_decompressed_size: Some(777),
      local: false,
      cam_json: false,
      units: None,
    };
    let limits = build_limits(&cli);
    assert_eq!(limits.max_file_size, 999);
    assert_eq!(limits.max_stream_size, 888);
    assert_eq!(limits.max_decompressed_size, 777);
  }
}
