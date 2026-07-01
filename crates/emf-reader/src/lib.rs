//! Parse EMF records and replay graphics state into Drawing IR.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod parser;
mod record;
mod replay;

pub use error::{EmfError, EmfResult};
pub use parser::{
  EmfDocument, EmfRecord, RecordClass, DEFAULT_MAX_RECORD_COUNT, DEFAULT_MAX_RECORD_SIZE,
};
pub use replay::replay_to_drawing;
