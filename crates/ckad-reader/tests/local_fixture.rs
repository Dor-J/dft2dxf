//! Integration tests against gitignored local cncKad fixtures.

use std::path::PathBuf;

use ckad_reader::read_to_drawing;

fn local_fixture(name: &str) -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../../tests/fixtures/valid/local")
    .join(name)
}

#[test]
fn parses_local_cnckad_fixtures_when_present() {
  let dir = local_fixture("");
  if !dir.is_dir() {
    return;
  }

  let mut tested = 0usize;
  for entry in std::fs::read_dir(dir.parent().unwrap()).unwrap().flatten() {
    let path = entry.path();
    if path.extension().and_then(|ext| ext.to_str()) != Some("DFT") {
      continue;
    }
    if !path.starts_with(&dir) {
      continue;
    }
    let drawing = read_to_drawing(&path, ckad_reader::DEFAULT_MAX_FILE_SIZE)
      .unwrap_or_else(|err| panic!("{} should parse: {err}", path.display()));
    assert!(
      !drawing.sheets.is_empty() && !drawing.sheets[0].entities.is_empty(),
      "{} should contain geometry",
      path.display()
    );
    tested += 1;
  }

  if tested == 0 {
    eprintln!("skip: no local .DFT fixtures under tests/fixtures/valid/local/");
  }
}
