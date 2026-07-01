//! Read Solid Edge `.dft` draft files as Compound File Binary containers and
//! extract embedded viewer EMF sheet streams.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod cfb;
mod decompress;
mod document;
mod error;
mod limits;
mod metadata;
mod sheet;
mod storage;

pub use document::{DftDocument, DftOpenOptions, InspectReport};
pub use error::{DftError, DftResult};
pub use limits::Limits;
pub use metadata::{DraftDocumentInfo, PaperUnit, SheetInfo, StorageEntry, StorageEntryKind, StorageTree};
pub use sheet::{ExtractedEmf, Sheet};
pub use storage::parse_viewer_document_info;
pub use decompress::decompress_zlib_bounded;
