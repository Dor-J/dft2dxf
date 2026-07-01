//! Synthetic cncKad text `.dft` content for tests.

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

/// Returns a richer cncKad fixture covering metadata, geometry, and CAM.
#[must_use]
pub fn professional_cnckad_dft() -> String {
  [
    "gKad 9.80",
    "cncKad Version 95276",
    "None",
    "",
    "[100]",
    "PRO-PART",
    "CUSTOMER",
    "",
    "[200]",
    "/E 0 0 200 100",
    "/P 200 100 1000 25",
    "/M 4 0 1",
    "",
    "[210]",
    "KFactor 0.400000",
    "",
    "[300]",
    "LINES",
    "1",
    "0 0 200 0 2 0 0 0 -1 0 0",
    "1 0 100 0 15 0",
    "POINTS",
    "0",
    "CIRCLES",
    "1",
    "100 50 10 15 0 1 0 0 0 -1 0",
    "ARCS",
    "1",
    "10 10 5 15 0 3 0 0 0 -1 0",
    "10 15 5 15",
    "0 90",
    "",
    "[310]",
    "LINES",
    "1",
    "0 100 200 100 2 0 0 0 -1 0 0",
    "1 0 200 0 12 0",
    "POINTS",
    "0",
    "CIRCLES",
    "0",
    "ARCS",
    "0",
    "",
    "[503]",
    "1.500000",
    "",
    "[1100]",
    "2 5",
    "R 50 5",
    "1 0 0",
    "TOOLCM \"(M)\"",
    "C 10",
    "1 0 0",
    "TOOLCM \" punch\"",
    "",
    "[1200]",
    "ONLINE",
    "1 0 0 0 0 0 0 0 6",
    "0 0 2 1 10 10 190 10 0 0 1 1 4 0 0 0 0",
    "0 0 0 0 0 0 0 0",
    "",
    "SINGLE",
    "0 0 0 0 0 0 0 0 5",
    "0 0 10 0 100 50 -1 1 0",
    "0 0 0 0 0 0 0 1 1 0 0 0 0 0",
    "",
    "OLE4DM extension line should be skipped in geometry only",
    "",
    "EOF",
  ]
  .join("\n")
}

/// Writes a minimal cncKad `.dft` file to `path`.
///
/// # Errors
///
/// Returns an I/O error if `path` cannot be written.
pub fn write_minimal_cnckad_dft(path: &std::path::Path) -> std::io::Result<()> {
  std::fs::write(path, minimal_cnckad_dft())
}

/// Writes the professional cncKad fixture to `path`.
///
/// # Errors
///
/// Returns an I/O error if `path` cannot be written.
pub fn write_professional_cnckad_dft(path: &std::path::Path) -> std::io::Result<()> {
  std::fs::write(path, professional_cnckad_dft())
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

/// Returns professional cncKad content encoded as UTF-16 LE (with BOM).
#[must_use]
pub fn professional_cnckad_dft_utf16_le() -> Vec<u8> {
  let mut bytes = vec![0xFF, 0xFE];
  for unit in professional_cnckad_dft().encode_utf16() {
    bytes.extend_from_slice(&unit.to_le_bytes());
  }
  bytes
}
