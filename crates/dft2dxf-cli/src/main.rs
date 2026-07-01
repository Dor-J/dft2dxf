//! `dft2dxf` binary entry point.

#![forbid(unsafe_code)]

use clap::Parser;
use dft2dxf_cli::{init_tracing, run, Cli};

fn main() -> anyhow::Result<()> {
  init_tracing();
  run(Cli::parse())
}
