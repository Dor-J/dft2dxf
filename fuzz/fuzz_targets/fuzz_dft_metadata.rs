#![no_main]

use libfuzzer_sys::fuzz_target;
use dft_reader::{parse_viewer_document_info, Limits};

fuzz_target!(|data: &[u8]| {
  let _ = parse_viewer_document_info(data, &Limits::strict());
});
