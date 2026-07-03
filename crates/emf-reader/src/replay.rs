//! Replay a subset of EMF drawing records into Drawing IR.

use std::collections::HashMap;

use drawing_ir::{
  ArcSegment, Color, Diagnostic, Entity, EntityKind, Path, PathSegment, Point, Polyline, Sheet,
  SourceProvenance, StrokeStyle, Style, TextRun,
};

use crate::parser::{EmfDocument, EmfRecord};
use crate::record::{
  EMR_ARC, EMR_ARCTO, EMR_BITBLT, EMR_CHORD, EMR_CREATEBRUSH, EMR_CREATEPEN, EMR_ELLIPSE,
  EMR_EXCLUDECLIPRECT, EMR_EXTCREATEFONTINDIRECTW, EMR_EXTCREATEPEN, EMR_EXTTEXTOUTA,
  EMR_EXTTEXTOUTW, EMR_INTERSECTCLIPRECT, EMR_LINETO, EMR_MODIFYWORLDTRANSFORM, EMR_MOVETOEX,
  EMR_PIE, EMR_POLYBEZIER16, EMR_POLYGON, EMR_POLYGON16, EMR_POLYLINE, EMR_POLYLINE16,
  EMR_RECTANGLE, EMR_SELECTOBJECT, EMR_SETMAPMODE, EMR_SETWORLDTRANSFORM, EMR_STRETCHDIBITS,
};

/// Replays supported EMF records into a single drawing sheet.
#[must_use]
#[allow(clippy::too_many_lines)]
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
  let mut pens: HashMap<u32, StrokeStyle> = HashMap::new();
  let mut fonts: HashMap<u32, f64> = HashMap::new();
  let mut scale_x = 1.0f64;
  let mut scale_y = 1.0f64;
  let mut text_height = 12.0f64;

  if let Some(message) = emf
    .header
    .record_count_mismatch(u32::try_from(emf.records.len()).unwrap_or(u32::MAX))
  {
    drawing.push_diagnostic(drawing_ir::Diagnostic::warning(
      "emf.header_record_count",
      message,
    ));
  }

  for record in &emf.records {
    match record.record_type {
      EMR_SETMAPMODE => {
        if let Some((sx, sy)) = parse_mapping_scale(record) {
          scale_x = sx;
          scale_y = sy;
        }
      }
      EMR_SETWORLDTRANSFORM | EMR_MODIFYWORLDTRANSFORM => {
        if let Some((sx, sy)) = parse_world_transform_scale(record) {
          scale_x *= sx;
          scale_y *= sy;
        }
      }
      EMR_CREATEPEN => {
        if let Some((index, pen)) = parse_create_pen(record) {
          pens.insert(index, pen);
        }
      }
      EMR_EXTCREATEPEN => {
        if let Some((index, pen)) = parse_ext_create_pen(record) {
          pens.insert(index, pen);
        }
      }
      EMR_EXTCREATEFONTINDIRECTW => {
        if let Some((index, height)) = parse_ext_create_font(record) {
          fonts.insert(index, height);
        }
      }
      EMR_SELECTOBJECT => {
        if let Some(index) = parse_select_object(record) {
          if let Some(pen) = pens.get(&index) {
            stroke = pen.clone();
          } else if let Some(height) = fonts.get(&index) {
            text_height = *height;
          } else {
            drawing.push_diagnostic(drawing_ir::Diagnostic::warning(
              "emf.invalid_object_index",
              format!("SELECTOBJECT references unknown index {index}"),
            ));
          }
        }
      }
      EMR_RECTANGLE => {
        if let Some((top_left, bottom_right)) = parse_rectangle(record, scale_x, scale_y) {
          sheet.entities.push(styled_entity(
            EntityKind::Rectangle {
              top_left,
              bottom_right,
            },
            &stroke,
            record,
          ));
        }
      }
      EMR_POLYLINE | EMR_POLYLINE16 => {
        if let Some(points) = parse_poly(record, scale_x, scale_y) {
          sheet.entities.push(styled_entity(
            EntityKind::Polyline(Polyline {
              points,
              closed: false,
            }),
            &stroke,
            record,
          ));
        }
      }
      EMR_POLYGON | EMR_POLYGON16 => {
        if let Some(points) = parse_poly(record, scale_x, scale_y) {
          sheet.entities.push(styled_entity(
            EntityKind::Polyline(Polyline {
              points,
              closed: true,
            }),
            &stroke,
            record,
          ));
        }
      }
      EMR_POLYBEZIER16 => {
        if let Some(path) = parse_polybezier16(record, scale_x, scale_y) {
          sheet
            .entities
            .push(styled_entity(EntityKind::Path(path), &stroke, record));
        }
      }
      EMR_ARC | EMR_ARCTO | EMR_CHORD | EMR_PIE => {
        if let Some(arc) = parse_arc(record, scale_x, scale_y) {
          sheet
            .entities
            .push(styled_entity(EntityKind::Arc(arc), &stroke, record));
        }
      }
      EMR_ELLIPSE => {
        if let Some(circle) = parse_ellipse(record, scale_x, scale_y) {
          sheet.entities.push(styled_entity(
            EntityKind::Circle {
              center: circle.0,
              radius: circle.1,
            },
            &stroke,
            record,
          ));
        }
      }
      EMR_MOVETOEX => {
        if let Some(point) = parse_point_record(record, scale_x, scale_y) {
          current_point = point;
        }
      }
      EMR_LINETO => {
        if let Some(to) = parse_point_record(record, scale_x, scale_y) {
          let from = current_point;
          sheet.entities.push(styled_entity(
            EntityKind::Line { from, to },
            &stroke,
            record,
          ));
          current_point = to;
        }
      }
      EMR_EXTTEXTOUTA | EMR_EXTTEXTOUTW => {
        if let Some(text) = parse_text_record(record, scale_x, scale_y) {
          sheet.entities.push(styled_entity(
            EntityKind::Text(TextRun {
              position: text.0,
              text: text.1,
              font_family: None,
              font_size: text_height,
              rotation_deg: 0.0,
              style: Style {
                stroke: Some(stroke.clone()),
                fill: None,
              },
              provenance: Some(provenance(record)),
            }),
            &stroke,
            record,
          ));
        }
      }
      EMR_INTERSECTCLIPRECT | EMR_EXCLUDECLIPRECT => {
        drawing.push_diagnostic(drawing_ir::Diagnostic::warning(
          "emf.clipping_unsupported",
          format!(
            "clipping record 0x{:08X} at index {}",
            record.record_type, record.index
          ),
        ));
      }
      EMR_CREATEBRUSH => {
        drawing.push_diagnostic(drawing_ir::Diagnostic::warning(
          "emf.fill_unsupported",
          format!("brush record at index {}", record.index),
        ));
      }
      EMR_BITBLT | EMR_STRETCHDIBITS => {
        drawing.push_diagnostic(drawing_ir::Diagnostic::warning(
          "emf.raster_unsupported",
          format!(
            "raster record 0x{:08X} at index {}",
            record.record_type, record.index
          ),
        ));
      }
      other if record.class() == crate::parser::RecordClass::Unsupported => {
        drawing.push_diagnostic(diagnostic_for_unsupported_record(other, record.index));
      }
      _ => {}
    }
  }

  build_paths_from_moveto_lineto(&mut sheet);

  sheet.recompute_bounds();
  drawing.sheets.push(sheet);
  drawing
}

fn diagnostic_for_unsupported_record(record_type: u32, record_index: u32) -> Diagnostic {
  Diagnostic::unsupported_record(record_type, record_index)
}

fn styled_entity(kind: EntityKind, stroke: &StrokeStyle, record: &EmfRecord) -> Entity {
  Entity {
    layer: None,
    style: Style {
      stroke: Some(stroke.clone()),
      fill: None,
    },
    kind,
    provenance: Some(provenance(record)),
  }
}

fn provenance(record: &EmfRecord) -> SourceProvenance {
  SourceProvenance {
    emf_record_index: Some(record.index),
    emf_record_type: Some(record.record_type),
  }
}

fn scale_point(point: Point, scale_x: f64, scale_y: f64) -> Point {
  Point::new(point.x * scale_x, point.y * scale_y)
}

fn read_u32(record: &EmfRecord, offset: usize) -> Option<u32> {
  Some(u32::from_le_bytes(
    record.data.get(offset..offset + 4)?.try_into().ok()?,
  ))
}

fn read_i16_as_f64(record: &EmfRecord, offset: usize) -> Option<f64> {
  Some(f64::from(i16::from_le_bytes(
    record.data.get(offset..offset + 2)?.try_into().ok()?,
  )))
}

fn read_i32_as_f64(record: &EmfRecord, offset: usize) -> Option<f64> {
  Some(f64::from(i32::from_le_bytes(
    record.data.get(offset..offset + 4)?.try_into().ok()?,
  )))
}

fn read_f32_as_f64(record: &EmfRecord, offset: usize) -> Option<f64> {
  Some(f64::from(f32::from_le_bytes(
    record.data.get(offset..offset + 4)?.try_into().ok()?,
  )))
}

fn parse_mapping_scale(record: &EmfRecord) -> Option<(f64, f64)> {
  if record.data.len() < 12 {
    return None;
  }
  let mode = read_u32(record, 8)?;
  if mode == 8 {
    return Some((0.0254, 0.0254));
  }
  None
}

fn parse_world_transform_scale(record: &EmfRecord) -> Option<(f64, f64)> {
  if record.data.len() < 40 {
    return None;
  }
  let m11 = read_f32_as_f64(record, 8)?;
  let m22 = read_f32_as_f64(record, 20)?;
  Some((m11.abs().max(1e-9), m22.abs().max(1e-9)))
}

fn parse_create_pen(record: &EmfRecord) -> Option<(u32, StrokeStyle)> {
  if record.data.len() < 28 {
    return None;
  }
  let index = read_u32(record, 8)?;
  let width = read_i32_as_f64(record, 16)?;
  let color = read_u32(record, 24)?;
  Some((index, stroke_from_color_ref(color, width.max(1.0))))
}

fn parse_ext_create_pen(record: &EmfRecord) -> Option<(u32, StrokeStyle)> {
  if record.data.len() < 32 {
    return None;
  }
  let index = read_u32(record, 8)?;
  let width = f64::from(read_u32(record, 16)?);
  let color = read_u32(record, 24)?;
  Some((index, stroke_from_color_ref(color, width.max(1.0))))
}

fn parse_ext_create_font(record: &EmfRecord) -> Option<(u32, f64)> {
  if record.data.len() < 20 {
    return None;
  }
  let index = read_u32(record, 8)?;
  let height = i32::from_le_bytes(record.data[12..16].try_into().ok()?);
  Some((index, f64::from(height.unsigned_abs())))
}

fn parse_select_object(record: &EmfRecord) -> Option<u32> {
  if record.data.len() < 12 {
    return None;
  }
  read_u32(record, 8)
}

fn stroke_from_color_ref(color_ref: u32, width: f64) -> StrokeStyle {
  StrokeStyle {
    color: Color::rgb(
      (color_ref & 0xFF) as u8,
      ((color_ref >> 8) & 0xFF) as u8,
      ((color_ref >> 16) & 0xFF) as u8,
    ),
    width,
  }
}

fn parse_point_record(record: &EmfRecord, scale_x: f64, scale_y: f64) -> Option<Point> {
  if record.data.len() < 16 {
    return None;
  }
  let x = read_i32_as_f64(record, 8)?;
  let y = read_i32_as_f64(record, 12)?;
  Some(scale_point(Point::new(x, y), scale_x, scale_y))
}

fn parse_rectangle(record: &EmfRecord, scale_x: f64, scale_y: f64) -> Option<(Point, Point)> {
  if record.data.len() < 24 {
    return None;
  }
  let left = read_i32_as_f64(record, 8)?;
  let top = read_i32_as_f64(record, 12)?;
  let right = read_i32_as_f64(record, 16)?;
  let bottom = read_i32_as_f64(record, 20)?;
  Some((
    scale_point(Point::new(left, top), scale_x, scale_y),
    scale_point(Point::new(right, bottom), scale_x, scale_y),
  ))
}

fn parse_poly(record: &EmfRecord, scale_x: f64, scale_y: f64) -> Option<Vec<Point>> {
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
      let x = read_i16_as_f64(record, offset)?;
      let y = read_i16_as_f64(record, offset + 2)?;
      points.push(scale_point(Point::new(x, y), scale_x, scale_y));
      offset += 4;
    } else {
      let x = read_i32_as_f64(record, offset)?;
      let y = read_i32_as_f64(record, offset + 4)?;
      points.push(scale_point(Point::new(x, y), scale_x, scale_y));
      offset += 8;
    }
  }
  Some(points)
}

fn parse_polybezier16(record: &EmfRecord, scale_x: f64, scale_y: f64) -> Option<Path> {
  let points = parse_poly(record, scale_x, scale_y)?;
  if points.len() < 4 {
    return None;
  }
  let mut segments = vec![PathSegment::MoveTo { to: points[0] }];
  for chunk in points[1..].chunks(3) {
    if chunk.len() < 3 {
      break;
    }
    segments.push(PathSegment::LineTo { to: chunk[2] });
  }
  Some(Path { segments })
}

fn parse_arc(record: &EmfRecord, scale_x: f64, scale_y: f64) -> Option<ArcSegment> {
  if record.data.len() < 40 {
    return None;
  }
  let left = read_i32_as_f64(record, 8)?;
  let top = read_i32_as_f64(record, 12)?;
  let right = read_i32_as_f64(record, 16)?;
  let bottom = read_i32_as_f64(record, 20)?;
  let start_x = read_i32_as_f64(record, 24)?;
  let start_y = read_i32_as_f64(record, 28)?;
  let end_x = read_i32_as_f64(record, 32)?;
  let end_y = read_i32_as_f64(record, 36)?;

  let center = scale_point(
    Point::new((left + right) * 0.5, (top + bottom) * 0.5),
    scale_x,
    scale_y,
  );
  let radius =
    ((right - left).abs().max((bottom - top).abs()) * 0.5) * scale_x.abs().max(scale_y.abs());
  let start = scale_point(Point::new(start_x, start_y), scale_x, scale_y);
  let end = scale_point(Point::new(end_x, end_y), scale_x, scale_y);
  let start_angle = (start.y - center.y).atan2(start.x - center.x);
  let mut end_angle = (end.y - center.y).atan2(end.x - center.x);
  if end_angle <= start_angle {
    end_angle += std::f64::consts::TAU;
  }
  Some(ArcSegment {
    center,
    radius,
    start_angle,
    end_angle,
  })
}

fn parse_ellipse(record: &EmfRecord, scale_x: f64, scale_y: f64) -> Option<(Point, f64)> {
  if record.data.len() < 24 {
    return None;
  }
  let left = read_i32_as_f64(record, 8)?;
  let top = read_i32_as_f64(record, 12)?;
  let right = read_i32_as_f64(record, 16)?;
  let bottom = read_i32_as_f64(record, 20)?;
  let center = scale_point(
    Point::new((left + right) * 0.5, (top + bottom) * 0.5),
    scale_x,
    scale_y,
  );
  let radius =
    ((right - left).abs().max((bottom - top).abs()) * 0.5) * scale_x.abs().max(scale_y.abs());
  Some((center, radius))
}

fn parse_text_record(record: &EmfRecord, scale_x: f64, scale_y: f64) -> Option<(Point, String)> {
  if record.data.len() < 52 {
    return None;
  }
  let x = read_i32_as_f64(record, 36)?;
  let y = read_i32_as_f64(record, 40)?;
  let n_chars = read_u32(record, 44)?;
  let off_string = usize::try_from(read_u32(record, 48)?).ok()?;
  if off_string >= record.data.len() {
    return None;
  }
  let text = if record.record_type == EMR_EXTTEXTOUTW {
    let byte_len = usize::try_from(n_chars).ok()?.saturating_mul(2);
    let end = off_string.saturating_add(byte_len).min(record.data.len());
    decode_utf16_le(record.data.get(off_string..end)?).unwrap_or_default()
  } else {
    let byte_len = usize::try_from(n_chars).ok()?;
    let end = off_string.saturating_add(byte_len).min(record.data.len());
    String::from_utf8_lossy(record.data.get(off_string..end)?)
      .trim_end_matches('\0')
      .to_string()
  };
  if text.is_empty() {
    return None;
  }
  Some((scale_point(Point::new(x, y), scale_x, scale_y), text))
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

  let path_provenance = line_entities.first().and_then(|entity| entity.provenance.clone());
  let path_style = line_entities
    .first()
    .map(|entity| entity.style.clone())
    .unwrap_or_default();

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
      style: path_style,
      kind: EntityKind::Path(Path { segments }),
      provenance: path_provenance,
    });
  }
}
