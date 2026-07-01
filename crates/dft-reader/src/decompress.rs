//! Bounded zlib decompression for embedded sheet streams.

use std::io::Read;

use flate2::read::ZlibDecoder;

use crate::error::{DftError, DftResult};
use crate::limits::Limits;

/// Decompresses zlib-wrapped data with output and input size limits.
///
/// # Errors
///
/// Returns [`DftError::LimitExceeded`] when input or output exceeds configured limits, or
/// [`DftError::DecompressionFailed`] when the zlib stream is invalid.
pub fn decompress_zlib_bounded(
  compressed: &[u8],
  stream_name: &str,
  limits: &Limits,
) -> DftResult<Vec<u8>> {
  decompress_zlib(compressed, stream_name, limits)
}

/// Decompresses zlib-wrapped data with output and input size limits.
pub(crate) fn decompress_zlib(
  compressed: &[u8],
  stream_name: &str,
  limits: &Limits,
) -> DftResult<Vec<u8>> {
  let input_len = compressed.len() as u64;
  if input_len > limits.max_stream_size {
    return Err(DftError::limit(
      "max_stream_size",
      limits.max_stream_size,
      input_len,
    ));
  }

  let mut decoder = ZlibDecoder::new(compressed);
  let mut output = Vec::new();
  let mut buffer = [0u8; 8192];

  loop {
    let read = decoder
      .read(&mut buffer)
      .map_err(|err| DftError::DecompressionFailed {
        stream: stream_name.to_string(),
        message: err.to_string(),
      })?;
    if read == 0 {
      break;
    }
    let new_len = output
      .len()
      .checked_add(read)
      .ok_or_else(|| DftError::DecompressionFailed {
        stream: stream_name.to_string(),
        message: "decompressed size overflow".to_string(),
      })?;
    if new_len as u64 > limits.max_decompressed_size {
      return Err(DftError::limit(
        "max_decompressed_size",
        limits.max_decompressed_size,
        new_len as u64,
      ));
    }
    output.extend_from_slice(&buffer[..read]);
  }

  Ok(output)
}
