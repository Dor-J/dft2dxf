use assert_cmd::Command;
use dft2dxf_testkit::{build_minimal_dft, build_rectangle_emf, MinimalDftSpec};
use predicates::prelude::*;

#[test]
fn inspect_command_lists_synthetic_sheet() {
  let dir = tempfile::tempdir().unwrap();
  let dft_path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 100, 50);
  build_minimal_dft(&dft_path, &MinimalDftSpec::one_sheet("Sheet1", emf)).unwrap();

  Command::cargo_bin("dft2dxf")
    .unwrap()
    .args(["inspect", dft_path.to_str().unwrap()])
    .assert()
    .success()
    .stdout(predicate::str::contains("Sheet1"));
}

#[test]
fn extract_emf_writes_file() {
  let dir = tempfile::tempdir().unwrap();
  let dft_path = dir.path().join("sample.dft");
  let out_dir = dir.path().join("emf");
  let emf = build_rectangle_emf(0, 0, 100, 50);
  build_minimal_dft(&dft_path, &MinimalDftSpec::one_sheet("Sheet1", emf)).unwrap();

  Command::cargo_bin("dft2dxf")
    .unwrap()
    .args([
      "extract-emf",
      dft_path.to_str().unwrap(),
      "--output-dir",
      out_dir.to_str().unwrap(),
      "--sheet",
      "1",
    ])
    .assert()
    .success();

  assert!(out_dir.join("sheet-1.emf").exists());
}

#[test]
fn validate_command_succeeds_for_synthetic_dft() {
  let dir = tempfile::tempdir().unwrap();
  let dft_path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 100, 50);
  build_minimal_dft(&dft_path, &MinimalDftSpec::one_sheet("Sheet1", emf)).unwrap();

  Command::cargo_bin("dft2dxf")
    .unwrap()
    .args(["validate", dft_path.to_str().unwrap()])
    .assert()
    .success()
    .stdout(predicate::str::contains("validation ok"));
}

#[test]
fn inspect_json_output_contains_sheet_metadata() {
  let dir = tempfile::tempdir().unwrap();
  let dft_path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 100, 50);
  build_minimal_dft(&dft_path, &MinimalDftSpec::one_sheet("Sheet1", emf)).unwrap();

  Command::cargo_bin("dft2dxf")
    .unwrap()
    .args([
      "inspect",
      dft_path.to_str().unwrap(),
      "--format",
      "json",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"has_viewer_info\": true"));
}
