//! Canonical vector drawing intermediate representation.

#![warn(missing_docs)]

mod diagnostic;
mod drawing;
mod entity;
mod geometry;
mod style;

pub use diagnostic::{Diagnostic, DiagnosticSeverity, SourceProvenance};
pub use drawing::{Drawing, Sheet};
pub use entity::{Entity, EntityKind, TextRun};
pub use geometry::{ArcSegment, BoundingBox, Path, PathSegment, Point, Polyline};
pub use style::{FillStyle, StrokeStyle, Style};
