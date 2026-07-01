//! Minimal synthetic cncKad text `.dft` content for tests.

/// Returns a minimal valid cncKad `.dft` document with one line entity.
#[must_use]
pub fn minimal_cnckad_dft() -> String {
  [
    "gKad 9.80",
    "cncKad Version 95276",
    "None",
    "",
    "[100]",
    "TEST-PART",
    "",
    "[200]",
    "/E 0 0 100 50",
    "",
    "[300]",
    "LINES",
    "1",
    "0 0 100 0 2 0 0 0 -1 0 0",
    "1 0 100 0 15 0",
    "POINTS",
    "0",
    "CIRCLES",
    "0",
    "ARCS",
    "0",
    "",
    "EOF",
  ]
  .join("\n")
}

/// Writes a minimal cncKad `.dft` file to `path`.
pub fn write_minimal_cnckad_dft(path: &std::path::Path) -> std::io::Result<()> {
  std::fs::write(path, minimal_cnckad_dft())
}

/// Returns minimal cncKad content encoded as UTF-16 LE (with BOM), as seen in newer exports.
#[must_use]
pub fn minimal_cnckad_dft_utf16_le() -> Vec<u8> {
  let mut bytes = vec![0xFF, 0xFE];
  for unit in minimal_cnckad_dft().encode_utf16() {
    bytes.extend_from_slice(&unit.to_le_bytes());
  }
  bytes
}
