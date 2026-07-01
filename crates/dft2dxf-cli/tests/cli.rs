use assert_cmd::Command;
use dft2dxf_testkit::{
  build_minimal_dft, build_rectangle_emf, minimal_cnckad_dft, professional_cnckad_dft,
  write_minimal_cnckad_dft, MinimalDftSpec,
};
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
    .args(["inspect", dft_path.to_str().unwrap(), "--format", "json"])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"has_viewer_info\": true"));
}

#[test]
fn convert_cnckad_to_dxf_and_svg() {
  let dir = tempfile::tempdir().unwrap();
  let dft_path = dir.path().join("part.dft");
  write_minimal_cnckad_dft(&dft_path).unwrap();
  let dxf_path = dir.path().join("out.dxf");
  let svg_dir = dir.path().join("svg");

  Command::cargo_bin("dft2dxf")
    .unwrap()
    .args([
      "convert",
      dft_path.to_str().unwrap(),
      "--output",
      dxf_path.to_str().unwrap(),
      "--svg-preview",
      svg_dir.to_str().unwrap(),
    ])
    .assert()
    .success();

  assert!(dxf_path.exists());
  assert!(svg_dir.join("sheet-1.svg").exists());
}

#[test]
fn convert_solid_edge_synthetic() {
  let dir = tempfile::tempdir().unwrap();
  let dft_path = dir.path().join("sample.dft");
  let emf = build_rectangle_emf(0, 0, 100, 50);
  build_minimal_dft(&dft_path, &MinimalDftSpec::one_sheet("Sheet1", emf)).unwrap();
  let dxf_path = dir.path().join("out.dxf");

  Command::cargo_bin("dft2dxf")
    .unwrap()
    .args([
      "convert",
      dft_path.to_str().unwrap(),
      "--output",
      dxf_path.to_str().unwrap(),
    ])
    .assert()
    .success();

  assert!(dxf_path.exists());
}

#[test]
fn convert_all_batch_two_files() {
  let dir = tempfile::tempdir().unwrap();
  let input_dir = dir.path().join("in");
  std::fs::create_dir_all(&input_dir).unwrap();
  write_minimal_cnckad_dft(&input_dir.join("cnckad.dft")).unwrap();
  let emf = build_rectangle_emf(0, 0, 50, 50);
  build_minimal_dft(
    &input_dir.join("solid_edge.dft"),
    &MinimalDftSpec::one_sheet("S1", emf),
  )
  .unwrap();

  let dxf_dir = dir.path().join("dxf");
  let svg_dir = dir.path().join("svg");

  Command::cargo_bin("dft2dxf")
    .unwrap()
    .args([
      "convert-all",
      "--input-dir",
      input_dir.to_str().unwrap(),
      "--dxf-dir",
      dxf_dir.to_str().unwrap(),
      "--svg-dir",
      svg_dir.to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("converted 2 file(s)"));
}

#[test]
fn convert_all_cam_json_sidecar() {
  let dir = tempfile::tempdir().unwrap();
  let input_dir = dir.path().join("in");
  std::fs::create_dir_all(&input_dir).unwrap();
  std::fs::write(
    input_dir.join("pro.dft"),
    professional_cnckad_dft(),
  )
  .unwrap();
  let dxf_dir = dir.path().join("dxf");
  let svg_dir = dir.path().join("svg");

  Command::cargo_bin("dft2dxf")
    .unwrap()
    .args([
      "convert-all",
      "--input-dir",
      input_dir.to_str().unwrap(),
      "--dxf-dir",
      dxf_dir.to_str().unwrap(),
      "--svg-dir",
      svg_dir.to_str().unwrap(),
      "--cam-json",
    ])
    .assert()
    .success();

  assert!(dxf_dir.join("pro.cam.json").exists());
}

#[test]
fn inspect_cnckad_json() {
  let dir = tempfile::tempdir().unwrap();
  let dft_path = dir.path().join("part.dft");
  std::fs::write(&dft_path, minimal_cnckad_dft()).unwrap();

  Command::cargo_bin("dft2dxf")
    .unwrap()
    .args(["inspect", dft_path.to_str().unwrap(), "--format", "json"])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"sheets\""));
}

#[test]
fn validate_cnckad_file() {
  let dir = tempfile::tempdir().unwrap();
  let dft_path = dir.path().join("part.dft");
  std::fs::write(&dft_path, minimal_cnckad_dft()).unwrap();

  Command::cargo_bin("dft2dxf")
    .unwrap()
    .args(["validate", dft_path.to_str().unwrap()])
    .assert()
    .success()
    .stdout(predicate::str::contains("validation ok"));
}

#[test]
fn validate_fixtures_ci_directory() {
  let _ = dft2dxf_testkit::ensure_ci_fixtures();
  let fixtures = dft2dxf_testkit::discover_valid_dft_fixtures(false);
  assert!(
    fixtures.iter().any(|p| p.to_string_lossy().contains("ci")),
    "expected CI fixtures under tests/fixtures/valid/ci/"
  );

  Command::cargo_bin("dft2dxf")
    .unwrap()
    .arg("validate-fixtures")
    .assert()
    .success();
}
