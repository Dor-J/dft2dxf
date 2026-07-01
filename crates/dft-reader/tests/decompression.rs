use dft2dxf_testkit::{
  build_minimal_dft, build_rectangle_emf, invalid_zlib_payload, MinimalDftSpec, SheetSpec,
};
use dft_reader::{decompress_zlib_bounded, DftDocument, DftError, Limits};

#[test]
fn rejects_invalid_zlib_stream() {
  let err =
    decompress_zlib_bounded(&invalid_zlib_payload(), "test", &Limits::strict()).unwrap_err();
  assert!(matches!(err, DftError::DecompressionFailed { .. }));
}

#[test]
fn rejects_decompression_over_limit() {
  let mut limits = Limits::strict();
  limits.max_decompressed_size = 8;
  let payload = zlib_compress(b"0123456789abcdef");
  let err = decompress_zlib_bounded(&payload, "test", &limits).unwrap_err();
  assert!(matches!(err, DftError::LimitExceeded { .. }));
}

#[test]
fn rejects_invalid_sheet_zlib_in_dft() {
  let dir = tempfile::tempdir().unwrap();
  let path = dir.path().join("bad-zlib.dft");
  let emf = build_rectangle_emf(0, 0, 10, 10);
  let spec = MinimalDftSpec {
    sheets: vec![SheetSpec {
      name: "S".to_string(),
      width: 297.0,
      height: 210.0,
      emf,
      compressed_override: Some(invalid_zlib_payload()),
    }],
  };
  build_minimal_dft(&path, &spec).unwrap();

  let mut document = DftDocument::open(&path).unwrap();
  let err = document.extract_emf(1).unwrap_err();
  assert!(matches!(err, DftError::DecompressionFailed { .. }));
}

fn zlib_compress(data: &[u8]) -> Vec<u8> {
  use flate2::write::ZlibEncoder;
  use flate2::Compression;
  use std::io::Write;
  let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
  encoder.write_all(data).unwrap();
  encoder.finish().unwrap()
}
