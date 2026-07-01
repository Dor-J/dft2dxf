//! Parse EMF records and replay graphics state into Drawing IR.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod parser;
mod record;
mod replay;

pub use error::{EmfError, EmfResult};
pub use parser::{EmfDocument, EmfRecord, RecordClass};
pub use replay::replay_to_drawing;
