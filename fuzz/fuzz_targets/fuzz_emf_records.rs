#![no_main]

use libfuzzer_sys::fuzz_target;
use emf_reader::{EmfDocument, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE};

fuzz_target!(|data: &[u8]| {
  let _ = EmfDocument::parse(data, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE);
});
