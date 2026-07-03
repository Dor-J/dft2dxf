//! Synthetic fixtures for dft2dxf tests.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod cnckad_fixture;
mod dft_fixture;
mod emf_fixture;
mod fixture_paths;
mod malformed_fixture;

pub use cnckad_fixture::{
  minimal_cnckad_dft, minimal_cnckad_dft_utf16_le, professional_cnckad_dft,
  professional_cnckad_dft_utf16_le, write_minimal_cnckad_dft, write_professional_cnckad_dft,
};
pub use dft_fixture::{build_minimal_dft, MinimalDftSpec, SheetSpec};
pub use emf_fixture::{
  build_arc_emf, build_ellipse_emf, build_emf_invalid_bounds, build_emf_records,
  build_emf_wrong_n_bytes, build_pen_and_line_emf, build_polygon_emf, build_polyline_emf,
  build_rectangle_emf, build_text_emf, build_transform_emf, is_emf,
};
pub use fixture_paths::{
  active_valid_fixtures_dir, ci_fixtures_dir, discover_valid_dft_fixtures, ensure_ci_fixtures,
  local_fixtures_dir, use_local_fixtures, valid_fixtures_dir, workspace_root,
};
pub use malformed_fixture::{
  excessive_sheet_count_metadata, invalid_zlib_payload, negative_sheet_count_metadata,
  too_short_metadata,
};
