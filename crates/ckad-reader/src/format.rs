//! DFT container format detection.

/// Magic bytes for Microsoft Compound File Binary Format.
pub const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// Known on-disk container for a `.dft` file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DftContainerFormat {
  /// Solid Edge OLE compound file with embedded EMF viewer streams.
  SolidEdgeCompound,
  /// Metalix cncKad text drawing format.
  CncKad,
}

/// Returns true when `bytes` begin with a cncKad text header.
#[must_use]
pub fn is_cnckad_bytes(bytes: &[u8]) -> bool {
  detect_format(bytes) == Some(DftContainerFormat::CncKad)
}

/// Detects the container format from the first bytes of a `.dft` file.
#[must_use]
pub fn detect_format(bytes: &[u8]) -> Option<DftContainerFormat> {
  if bytes.len() >= CFB_MAGIC.len() && bytes[..CFB_MAGIC.len()] == CFB_MAGIC {
    return Some(DftContainerFormat::SolidEdgeCompound);
  }
  if is_cnckad_header_prefix(bytes) {
    return Some(DftContainerFormat::CncKad);
  }
  None
}

fn is_cnckad_header_prefix(bytes: &[u8]) -> bool {
  if is_cnckad_ascii_prefix(bytes) {
    return true;
  }
  is_cnckad_utf16_le_prefix(bytes)
}

fn is_cnckad_ascii_prefix(bytes: &[u8]) -> bool {
  if bytes.len() < 4 {
    return false;
  }
  // Observed files use "gKad"; some docs reference "CKad".
  let prefix = &bytes[..4];
  prefix == b"gKad" || prefix == b"CKad"
}

fn is_cnckad_utf16_le_prefix(bytes: &[u8]) -> bool {
  if bytes.len() < 10 || bytes[0..2] != [0xFF, 0xFE] {
    return false;
  }
  let prefix = &bytes[2..10];
  prefix == [0x67, 0x00, 0x4B, 0x00, 0x61, 0x00, 0x64, 0x00]
    || prefix == [0x43, 0x00, 0x4B, 0x00, 0x61, 0x00, 0x64, 0x00]
}
