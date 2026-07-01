//! Canonical vector drawing intermediate representation.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod cam;
mod diagnostic;
mod drawing;
mod entity;
mod geometry;
mod metadata;
mod style;

pub use cam::{CamOperation, CamProgram, CamTool};
pub use diagnostic::{Diagnostic, DiagnosticSeverity, SourceProvenance};
pub use drawing::{Drawing, Sheet};
pub use entity::{DimensionKind, Entity, EntityKind, TextRun};
pub use geometry::{ArcSegment, BoundingBox, Path, PathSegment, Point, Polyline};
pub use metadata::{DrawingMetadata, PaperUnit};
pub use style::{Color, FillStyle, StrokeStyle, Style};
