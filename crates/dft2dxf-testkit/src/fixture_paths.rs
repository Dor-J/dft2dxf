//! Paths and discovery for real Solid Edge `.dft` test fixtures.

use std::path::{Path, PathBuf};

/// Workspace root (`dft2dxf` repository root).
#[must_use]
pub fn workspace_root() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Default directory for redistributable valid fixtures (`tests/fixtures/valid/`).
#[must_use]
pub fn valid_fixtures_dir() -> PathBuf {
  workspace_root().join("tests/fixtures/valid")
}

/// Gitignored local-only fixtures (`tests/fixtures/valid/local/`).
#[must_use]
pub fn local_fixtures_dir() -> PathBuf {
  valid_fixtures_dir().join("local")
}

/// Returns the active fixtures directory.
#[must_use]
pub fn active_valid_fixtures_dir(use_local: bool) -> PathBuf {
  if use_local {
    local_fixtures_dir()
  } else {
    valid_fixtures_dir()
  }
}

/// Whether to load fixtures from the local (gitignored) directory.
///
/// Enabled when either:
/// - environment variable `DFT2DXF_LOCAL=1` is set, or
/// - `--local` / `-local` appears in the process arguments (e.g. `cargo test -- --local`).
#[must_use]
pub fn use_local_fixtures() -> bool {
  if std::env::var("DFT2DXF_LOCAL")
    .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
    .unwrap_or(false)
  {
    return true;
  }
  std::env::args().any(|arg| arg == "--local" || arg == "-local")
}

/// Committed CI fixtures (`tests/fixtures/valid/ci/`).
#[must_use]
pub fn ci_fixtures_dir() -> PathBuf {
  valid_fixtures_dir().join("ci")
}

/// Writes CI fixture files when missing (idempotent).
///
/// # Errors
///
/// Returns an I/O error if fixture directories or files cannot be created.
pub fn ensure_ci_fixtures() -> std::io::Result<()> {
  use crate::{build_minimal_dft, build_rectangle_emf, write_minimal_cnckad_dft, MinimalDftSpec};

  let dir = ci_fixtures_dir();
  std::fs::create_dir_all(&dir)?;
  let cnckad = dir.join("minimal_cnckad.dft");
  if !cnckad.is_file() {
    write_minimal_cnckad_dft(&cnckad)?;
  }
  let se = dir.join("minimal_solid_edge.dft");
  if !se.is_file() {
    let emf = build_rectangle_emf(0, 0, 100, 50);
    build_minimal_dft(&se, &MinimalDftSpec::one_sheet("CI-Sheet", emf))?;
  }
  Ok(())
}

/// Lists `.dft` files in the active valid fixtures directory and `valid/ci/`.
#[must_use]
pub fn discover_valid_dft_fixtures(use_local: bool) -> Vec<PathBuf> {
  if use_local {
    return discover_dft_files_in_dir(&local_fixtures_dir());
  }
  let mut files = discover_dft_files_in_dir(&valid_fixtures_dir());
  let ci = discover_dft_files_in_dir(&ci_fixtures_dir());
  files.extend(ci);
  files.sort();
  files.dedup();
  files
}

fn discover_dft_files_in_dir(dir: &Path) -> Vec<PathBuf> {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return Vec::new();
  };

  let mut files: Vec<PathBuf> = entries
    .filter_map(Result::ok)
    .map(|entry| entry.path())
    .filter(|path| path.is_file() && is_dft_file(path))
    .collect();
  files.sort();
  files
}

fn is_dft_file(path: &Path) -> bool {
  path
    .extension()
    .and_then(|value| value.to_str())
    .is_some_and(|ext| ext.eq_ignore_ascii_case("dft"))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn valid_fixtures_dir_points_at_repository_tree() {
    let dir = valid_fixtures_dir();
    assert!(dir.ends_with("tests/fixtures/valid"));
    assert!(dir.join("INTAKE.md").is_file());
  }

  #[test]
  fn local_fixtures_dir_is_under_valid() {
    let local = local_fixtures_dir();
    assert!(local.ends_with("tests/fixtures/valid/local"));
  }

  #[test]
  fn discover_skips_non_dft_extensions() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("note.txt"), b"x").unwrap();
    std::fs::write(dir.path().join("sample.DFT"), b"x").unwrap();
    let found = discover_dft_files_in_dir(dir.path());
    assert_eq!(found.len(), 1);
    assert!(found[0].to_string_lossy().contains("sample.DFT"));
  }

  #[test]
  fn ensure_ci_fixtures_writes_solid_edge_file() {
    let _ = ensure_ci_fixtures();
    let se = ci_fixtures_dir().join("minimal_solid_edge.dft");
    assert!(se.is_file(), "expected generated SE CI fixture");
  }
}
