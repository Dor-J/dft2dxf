//! Synthetic fixtures for dft2dxf tests.

#![warn(missing_docs)]

mod dft_fixture;
mod emf_fixture;
mod malformed_fixture;

pub use dft_fixture::{build_minimal_dft, MinimalDftSpec, SheetSpec};
pub use emf_fixture::{build_line_emf, build_rectangle_emf, is_emf};
pub use malformed_fixture::{
  excessive_sheet_count_metadata, invalid_zlib_payload, negative_sheet_count_metadata,
  too_short_metadata,
};
