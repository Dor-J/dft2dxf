//! Deterministic SVG serialization for Drawing IR.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod writer;

pub use error::{SvgError, SvgResult};
pub use writer::{write_drawing_to_file, write_drawing_to_string};
