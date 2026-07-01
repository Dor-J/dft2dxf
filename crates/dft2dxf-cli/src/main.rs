//! `dft2dxf` command-line interface.

#![forbid(unsafe_code)]

mod output;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use dft_reader::{DftDocument, DftOpenOptions, Limits};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(
  name = "dft2dxf",
  version,
  about = "Convert Solid Edge .dft draft files to portable vector outputs",
  long_about = "Extracts embedded EMF viewer data from Solid Edge .dft files and converts \
                visible vector representation to DXF/SVG. This tool does not recover native \
                Solid Edge CAD semantics."
)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,

  /// Input `.dft` file (shorthand for convert when used with --output).
  #[arg(value_name = "INPUT")]
  input: Option<PathBuf>,

  /// Output `.dxf` file (experimental convert shorthand).
  #[arg(short, long, value_name = "FILE")]
  output: Option<PathBuf>,

  /// One-based sheet index.
  #[arg(long, value_name = "INDEX")]
  sheet: Option<u32>,

  /// Output format for structured diagnostics.
  #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
  format: OutputFormat,

  /// Override maximum input file size in bytes.
  #[arg(long)]
  max_file_size: Option<u64>,

  /// Override maximum compressed stream size in bytes.
  #[arg(long)]
  max_stream_size: Option<u64>,

  /// Override maximum decompressed EMF size in bytes.
  #[arg(long)]
  max_decompressed_size: Option<u64>,
}

#[derive(Debug, Subcommand)]
enum Commands {
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
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
  /// Human-readable text output.
  Human,
  /// Machine-readable JSON output.
  Json,
}

fn main() -> Result<()> {
  init_tracing();
  let cli = Cli::parse();
  let limits = build_limits(&cli);

  match cli.command {
    Some(Commands::Inspect { input }) => cmd_inspect(&input, limits, cli.format),
    Some(Commands::ExtractEmf {
      input,
      output_dir,
      sheet,
    }) => cmd_extract_emf(&input, &output_dir, sheet, limits, cli.format),
    Some(Commands::Validate { input }) => cmd_validate(&input, limits, cli.format),
    Some(Commands::Convert {
      input,
      output,
      sheet,
      svg_preview,
    }) => cmd_convert(&input, &output, sheet, svg_preview, limits),
    None => {
      let input = cli
        .input
        .context("missing input .dft file; pass a path or use a subcommand")?;
      let output = cli
        .output
        .context("missing --output; use `dft2dxf convert` for explicit conversion")?;
      cmd_convert(&input, &output, cli.sheet, None, limits)
    }
  }
}

fn init_tracing() {
  let _ = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .try_init();
}

fn build_limits(cli: &Cli) -> Limits {
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
  let mut document =
    DftDocument::open_with_options(input, DftOpenOptions::new().with_limits(limits))?;
  let report = document.inspect().context("inspect failed")?;
  output::render_inspect(&report, format)?;
  Ok(())
}

fn cmd_extract_emf(
  input: &PathBuf,
  output_dir: &PathBuf,
  sheet: Option<u32>,
  limits: Limits,
  format: OutputFormat,
) -> Result<()> {
  let mut document =
    DftDocument::open_with_options(input, DftOpenOptions::new().with_limits(limits))?;
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
  let mut document =
    DftDocument::open_with_options(input, DftOpenOptions::new().with_limits(limits))?;
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

fn cmd_convert(
  input: &PathBuf,
  output: &PathBuf,
  sheet: Option<u32>,
  svg_preview: Option<PathBuf>,
  limits: Limits,
) -> Result<()> {
  let mut document =
    DftDocument::open_with_options(input, DftOpenOptions::new().with_limits(limits))?;
  let sheets = document.sheets().context("failed to read sheets")?;
  let index = sheet.unwrap_or_else(|| sheets.first().map(|s| s.index).unwrap_or(1));
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

  let drawing = emf_reader::replay_to_drawing(
    &emf_doc,
    Some(sheet_meta.index),
    Some(sheet_meta.name.clone()),
    Some(sheet_meta.info.width),
    Some(sheet_meta.info.height),
  );

  drawing_dxf::write_drawing_to_file(&drawing, output)
    .with_context(|| format!("failed to write DXF to {}", output.display()))?;

  if let Some(dir) = svg_preview {
    let svg_path = dir.join(format!("sheet-{index}.svg"));
    drawing_svg::write_drawing_to_file(&drawing, &svg_path)
      .with_context(|| format!("failed to write SVG to {}", svg_path.display()))?;
  }

  tracing::info!(output = %output.display(), sheet = index, "conversion complete");
  Ok(())
}
