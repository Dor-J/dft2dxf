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

/// Lists `.dft` files directly under the chosen valid fixtures directory (non-recursive).
#[must_use]
pub fn discover_valid_dft_fixtures(use_local: bool) -> Vec<PathBuf> {
  let dir = active_valid_fixtures_dir(use_local);
  discover_dft_files_in_dir(&dir)
}

fn discover_dft_files_in_dir(dir: &Path) -> Vec<PathBuf> {
  let entries = match std::fs::read_dir(dir) {
    Ok(value) => value,
    Err(_) => return Vec::new(),
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
}
