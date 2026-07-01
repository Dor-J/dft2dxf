//! Replay a subset of EMF drawing records into Drawing IR.

use drawing_ir::{
  Diagnostic, Entity, EntityKind, Path, PathSegment, Point, Polyline, Sheet, SourceProvenance,
  StrokeStyle, Style, TextRun,
};

use crate::parser::{EmfDocument, EmfRecord};
use crate::record::{
  EMR_EXTTEXTOUTA, EMR_EXTTEXTOUTW, EMR_LINETO, EMR_MOVETOEX, EMR_POLYGON, EMR_POLYGON16,
  EMR_POLYLINE, EMR_POLYLINE16, EMR_RECTANGLE,
};

/// Replays supported EMF records into a single drawing sheet.
pub fn replay_to_drawing(
  emf: &EmfDocument,
  sheet_index: Option<u32>,
  sheet_name: Option<String>,
  width: Option<f64>,
  height: Option<f64>,
) -> drawing_ir::Drawing {
  let mut drawing = drawing_ir::Drawing::new();
  let mut sheet = Sheet {
    index: sheet_index,
    name: sheet_name,
    width,
    height,
    entities: Vec::new(),
    bounds: None,
  };

  let mut current_point = Point::new(0.0, 0.0);
  let mut stroke = StrokeStyle::default();

  for record in &emf.records {
    match record.record_type {
      EMR_RECTANGLE => {
        if let Some((top_left, bottom_right)) = parse_rectangle(record) {
          sheet.entities.push(Entity {
            layer: None,
            style: Style {
              stroke: Some(stroke.clone()),
              fill: None,
            },
            kind: EntityKind::Rectangle {
              top_left,
              bottom_right,
            },
            provenance: provenance(record),
          });
        }
      }
      EMR_POLYLINE | EMR_POLYLINE16 => {
        if let Some(points) = parse_poly(record, false) {
          sheet.entities.push(Entity {
            layer: None,
            style: Style {
              stroke: Some(stroke.clone()),
              fill: None,
            },
            kind: EntityKind::Polyline(Polyline {
              points,
              closed: false,
            }),
            provenance: provenance(record),
          });
        }
      }
      EMR_POLYGON | EMR_POLYGON16 => {
        if let Some(points) = parse_poly(record, true) {
          sheet.entities.push(Entity {
            layer: None,
            style: Style {
              stroke: Some(stroke.clone()),
              fill: None,
            },
            kind: EntityKind::Polyline(Polyline {
              points,
              closed: true,
            }),
            provenance: provenance(record),
          });
        }
      }
      EMR_MOVETOEX => {
        if let Some(point) = parse_point_record(record) {
          current_point = point;
        }
      }
      EMR_LINETO => {
        if let Some(to) = parse_point_record(record) {
          let from = current_point;
          sheet.entities.push(Entity {
            layer: None,
            style: Style {
              stroke: Some(stroke.clone()),
              fill: None,
            },
            kind: EntityKind::Line { from, to },
            provenance: provenance(record),
          });
          current_point = to;
        }
      }
      EMR_EXTTEXTOUTA | EMR_EXTTEXTOUTW => {
        if let Some(text) = parse_text_record(record) {
          sheet.entities.push(Entity {
            layer: None,
            style: Style {
              stroke: Some(stroke.clone()),
              fill: None,
            },
            kind: EntityKind::Text(TextRun {
              position: text.0,
              text: text.1,
              font_family: None,
              font_size: 12.0,
              style: Style {
                stroke: Some(stroke.clone()),
                fill: None,
              },
              provenance: provenance(record),
            }),
            provenance: provenance(record),
          });
        }
      }
      other if record.class() == crate::parser::RecordClass::Unsupported => {
        drawing.push_diagnostic(Diagnostic::unsupported_record(other, record.index));
      }
      _ => {}
    }
  }

  // If moveto/lineto sequences exist without explicit line entities, attempt path build.
  build_paths_from_moveto_lineto(&mut sheet);

  sheet.recompute_bounds();
  drawing.sheets.push(sheet);
  drawing
}

fn provenance(record: &EmfRecord) -> Option<SourceProvenance> {
  Some(SourceProvenance {
    emf_record_index: Some(record.index),
    emf_record_type: Some(record.record_type),
  })
}

fn parse_point_record(record: &EmfRecord) -> Option<Point> {
  if record.data.len() < 16 {
    return None;
  }
  let x = i32::from_le_bytes(record.data[8..12].try_into().ok()?) as f64;
  let y = i32::from_le_bytes(record.data[12..16].try_into().ok()?) as f64;
  Some(Point::new(x, y))
}

fn parse_rectangle(record: &EmfRecord) -> Option<(Point, Point)> {
  if record.data.len() < 24 {
    return None;
  }
  let left = i32::from_le_bytes(record.data[8..12].try_into().ok()?) as f64;
  let top = i32::from_le_bytes(record.data[12..16].try_into().ok()?) as f64;
  let right = i32::from_le_bytes(record.data[16..20].try_into().ok()?) as f64;
  let bottom = i32::from_le_bytes(record.data[20..24].try_into().ok()?) as f64;
  Some((Point::new(left, top), Point::new(right, bottom)))
}

fn parse_poly(record: &EmfRecord, _closed: bool) -> Option<Vec<Point>> {
  if record.data.len() < 16 {
    return None;
  }
  let count = u32::from_le_bytes(record.data[8..12].try_into().ok()?) as usize;
  let is_16 = matches!(record.record_type, EMR_POLYLINE16 | EMR_POLYGON16);
  let point_size = if is_16 { 4 } else { 8 };
  let start = 12usize;
  let needed = start.checked_add(count.checked_mul(point_size)?)?;
  if record.data.len() < needed {
    return None;
  }

  let mut points = Vec::with_capacity(count);
  let mut offset = start;
  for _ in 0..count {
    if is_16 {
      let x = i16::from_le_bytes(record.data[offset..offset + 2].try_into().ok()?) as f64;
      let y = i16::from_le_bytes(record.data[offset + 2..offset + 4].try_into().ok()?) as f64;
      points.push(Point::new(x, y));
      offset += 4;
    } else {
      let x = i32::from_le_bytes(record.data[offset..offset + 4].try_into().ok()?) as f64;
      let y = i32::from_le_bytes(record.data[offset + 4..offset + 8].try_into().ok()?) as f64;
      points.push(Point::new(x, y));
      offset += 8;
    }
  }
  Some(points)
}

fn parse_text_record(record: &EmfRecord) -> Option<(Point, String)> {
  if record.data.len() < 24 {
    return None;
  }
  let x = i32::from_le_bytes(record.data[8..12].try_into().ok()?) as f64;
  let y = i32::from_le_bytes(record.data[12..16].try_into().ok()?) as f64;
  let string_bytes = record.data.get(24..)?;
  let text = if record.record_type == EMR_EXTTEXTOUTW {
    decode_utf16_le(string_bytes).unwrap_or_default()
  } else {
    String::from_utf8_lossy(string_bytes)
      .trim_end_matches('\0')
      .to_string()
  };
  if text.is_empty() {
    return None;
  }
  Some((Point::new(x, y), text))
}

fn decode_utf16_le(bytes: &[u8]) -> Option<String> {
  if bytes.len() % 2 != 0 {
    return None;
  }
  let units: Vec<u16> = bytes
    .chunks_exact(2)
    .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
    .take_while(|unit| *unit != 0)
    .collect();
  String::from_utf16(&units).ok()
}

fn build_paths_from_moveto_lineto(sheet: &mut Sheet) {
  let line_entities: Vec<_> = sheet
    .entities
    .iter()
    .filter(|entity| matches!(entity.kind, EntityKind::Line { .. }))
    .cloned()
    .collect();
  if line_entities.len() < 2 {
    return;
  }

  let mut segments = Vec::new();
  for entity in line_entities {
    if let EntityKind::Line { from, to } = entity.kind {
      if segments.is_empty() {
        segments.push(PathSegment::MoveTo { to: from });
      }
      segments.push(PathSegment::LineTo { to });
    }
  }
  if !segments.is_empty() {
    sheet.entities.push(Entity {
      layer: None,
      style: Style::default(),
      kind: EntityKind::Path(Path { segments }),
      provenance: None,
    });
  }
}
