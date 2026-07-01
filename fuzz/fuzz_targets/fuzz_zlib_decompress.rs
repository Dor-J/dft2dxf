#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use dft_reader::{decompress_zlib_bounded, Limits};

fuzz_target!(|data: &[u8]| {
  let _ = decompress_zlib_bounded(data, "fuzz", &Limits::strict());
});
