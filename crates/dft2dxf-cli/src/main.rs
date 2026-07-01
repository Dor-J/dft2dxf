//! `dft2dxf` command-line interface.

#![forbid(unsafe_code)]

mod output;

use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ckad_reader::{detect_format, read_to_drawing, DftContainerFormat};
use clap::{Parser, Subcommand, ValueEnum};
use dft_reader::{DftDocument, DftOpenOptions, Limits};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(
  name = "dft2dxf",
  version,
  about = "Convert .dft draft files (Solid Edge or cncKad) to portable vector outputs",
  long_about = "Reads Solid Edge compound .dft files via embedded EMF viewer data, or \
                Metalix cncKad text .dft geometry, and converts visible vector representation \
                to DXF/SVG."
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

  /// Use gitignored local `.dft` fixtures under `tests/fixtures/valid/local/`.
  #[arg(short = 'l', long = "local", global = true)]
  local: bool,
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
    Some(Commands::ValidateFixtures) => cmd_validate_fixtures(limits, cli.local, cli.format),
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
  match sniff_format(input)? {
    DftContainerFormat::CncKad => cmd_inspect_cnckad(input, limits, format),
    DftContainerFormat::SolidEdgeCompound => {
      let mut document =
        DftDocument::open_with_options(input, DftOpenOptions::new().with_limits(limits))?;
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
  match sniff_format(input)? {
    DftContainerFormat::CncKad => cmd_validate_cnckad(input, limits, format),
    DftContainerFormat::SolidEdgeCompound => {
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
  }
}

fn cmd_convert(
  input: &PathBuf,
  output: &PathBuf,
  sheet: Option<u32>,
  svg_preview: Option<PathBuf>,
  limits: Limits,
) -> Result<()> {
  match sniff_format(input)? {
    DftContainerFormat::CncKad => {
      if sheet.is_some() {
        tracing::warn!("cncKad .dft files contain a single geometry sheet; --sheet is ignored");
      }
      cmd_convert_cnckad(input, output, svg_preview, limits)
    }
    DftContainerFormat::SolidEdgeCompound => {
      cmd_convert_solid_edge(input, output, sheet, svg_preview, limits)
    }
  }
}

fn cmd_convert_solid_edge(
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

  let mut drawing = emf_reader::replay_to_drawing(
    &emf_doc,
    Some(sheet_meta.index),
    Some(sheet_meta.name.clone()),
    Some(sheet_meta.info.width),
    Some(sheet_meta.info.height),
  );

  drawing_dxf::write_drawing_to_file(&mut drawing, output)
    .with_context(|| format!("failed to write DXF to {}", output.display()))?;

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
) -> Result<()> {
  let mut drawing = read_to_drawing(input, limits.max_file_size)
    .with_context(|| format!("failed to read cncKad file {}", input.display()))?;

  drawing_dxf::write_drawing_to_file(&mut drawing, output)
    .with_context(|| format!("failed to write DXF to {}", output.display()))?;

  if let Some(dir) = svg_preview {
    std::fs::create_dir_all(&dir)?;
    let svg_path = dir.join("sheet-1.svg");
    drawing_svg::write_drawing_to_file(&drawing, &svg_path)
      .with_context(|| format!("failed to write SVG to {}", svg_path.display()))?;
  }

  tracing::info!(output = %output.display(), format = "cnckad", "conversion complete");
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
  let entity_count: usize = drawing.sheets.iter().map(|sheet| sheet.entities.len()).sum();
  if entity_count == 0 {
    anyhow::bail!("no geometry entities found in cncKad file");
  }
  output::render_validate(drawing.sheets.len(), format)?;
  Ok(())
}

fn sniff_format(path: &Path) -> Result<DftContainerFormat> {
  let mut file = std::fs::File::open(path)
    .with_context(|| format!("failed to open {}", path.display()))?;
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
