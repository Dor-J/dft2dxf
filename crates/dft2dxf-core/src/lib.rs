//! In-memory `.dft` conversion for CLI and HTTP backends.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod convert;
mod error;
mod inspect;
mod validate;

pub use convert::{
  apply_units_override, build_cam_json, convert_bytes, load_drawing_from_bytes, ConvertOptions,
  ConvertOutput, ConvertSummary,
};
pub use error::{CoreError, CoreResult};
pub use inspect::{inspect_bytes, InspectOutput};
pub use validate::{validate_bytes, ValidateOutput};
