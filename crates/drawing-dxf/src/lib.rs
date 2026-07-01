//! Write Drawing IR to DXF.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod writer;

pub use error::{DxfError, DxfResult};
pub use writer::write_drawing_to_file;
