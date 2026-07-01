use dft2dxf_testkit::{build_line_emf, build_minimal_dft, is_emf, MinimalDftSpec};
use dft_reader::DftDocument;

#[test]
fn extracts_all_sheets_from_multi_sheet_dft() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("multi.dft");
  let emf_a = build_line_emf(0, 0, 50, 50);
  let emf_b = build_line_emf(10, 10, 90, 90);
  build_minimal_dft(
    &path,
    &MinimalDftSpec::multi_sheet(vec![("SheetA".to_string(), emf_a), ("SheetB".to_string(), emf_b)]),
  )
  .unwrap();

  let mut document = DftDocument::open(&path).unwrap();
  let sheets = document.sheets().unwrap();
  assert_eq!(sheets.len(), 2);
  assert_eq!(sheets[0].name, "SheetA");
  assert_eq!(sheets[1].name, "SheetB");

  let first = document.extract_emf(1).unwrap();
  let second = document.extract_emf(2).unwrap();
  assert!(is_emf(&first.data));
  assert!(is_emf(&second.data));
  assert_ne!(first.data, second.data);
}
