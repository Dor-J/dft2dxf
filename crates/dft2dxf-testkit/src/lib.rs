//! Synthetic fixtures for dft2dxf tests.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod cnckad_fixture;
mod dft_fixture;
mod emf_fixture;
mod fixture_paths;
mod malformed_fixture;

pub use cnckad_fixture::{minimal_cnckad_dft, minimal_cnckad_dft_utf16_le, write_minimal_cnckad_dft};
pub use dft_fixture::{build_minimal_dft, MinimalDftSpec, SheetSpec};
pub use emf_fixture::{build_rectangle_emf, is_emf};
pub use fixture_paths::{
  active_valid_fixtures_dir, discover_valid_dft_fixtures, local_fixtures_dir, use_local_fixtures,
  valid_fixtures_dir, workspace_root,
};
pub use malformed_fixture::{
  excessive_sheet_count_metadata, invalid_zlib_payload, negative_sheet_count_metadata,
  too_short_metadata,
};
