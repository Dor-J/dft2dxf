//! Read Metalix cncKad text `.dft` drawings into Drawing IR.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod cam;
mod error;
mod format;
mod metadata;
mod parser;
mod style;

pub use cam::parse_cam;
pub use error::{CkadError, CkadResult};
pub use format::{detect_format, is_cnckad_bytes, DftContainerFormat, CFB_MAGIC};
pub use parser::{parse_content, read_to_drawing, DEFAULT_MAX_FILE_SIZE};
