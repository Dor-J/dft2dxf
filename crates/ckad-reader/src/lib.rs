//! Read Metalix cncKad text `.dft` drawings into Drawing IR.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod format;
mod parser;

pub use error::{CkadError, CkadResult};
pub use format::{detect_format, is_cnckad_bytes, CFB_MAGIC, DftContainerFormat};
pub use parser::{parse_content, read_to_drawing, DEFAULT_MAX_FILE_SIZE};
