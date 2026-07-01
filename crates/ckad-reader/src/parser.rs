//! cncKad text `.dft` geometry parser.

use std::collections::BTreeMap;
use std::path::Path;

use drawing_ir::{
  Drawing, Entity, EntityKind, Point, Polyline, Sheet, Style,
};

use crate::error::{CkadError, CkadResult};

/// Default maximum input file size (50 MiB).
pub const DEFAULT_MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Polyline segments used when approximating circles.
const CIRCLE_SEGMENTS: usize = 32;

/// Polyline segments used when approximating arc spans.
const ARC_SEGMENTS: usize = 16;

/// Reads a cncKad text `.dft` file into [`Drawing`] IR.
pub fn read_to_drawing(path: &Path, max_file_size: u64) -> CkadResult<Drawing> {
  let bytes = std::fs::read(path).map_err(|source| CkadError::Io {
    path: path.to_path_buf(),
    source,
  })?;
  if bytes.len() as u64 > max_file_size {
    return Err(CkadError::FileTooLarge {
      limit: max_file_size,
      actual: bytes.len() as u64,
    });
  }
  if !crate::format::is_cnckad_bytes(&bytes) {
    return Err(CkadError::NotCncKad {
      message: "missing gKad/CKad header".to_string(),
    });
  }
  let content = decode_text(&bytes);
  parse_content(&content, Some(path.display().to_string()))
}

fn decode_text(bytes: &[u8]) -> String {
  if bytes.len() >= 2 && bytes[0..2] == [0xFF, 0xFE] {
    return decode_utf16_le(&bytes[2..]);
  }
  bytes.iter().map(|byte| *byte as char).collect()
}

fn decode_utf16_le(bytes: &[u8]) -> String {
  let units = bytes
    .chunks_exact(2)
    .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]));
  char::decode_utf16(units)
    .map(|unit| unit.unwrap_or('\u{FFFD}'))
    .collect()
}

/// Parses cncKad text content into [`Drawing`] IR.
pub fn parse_content(content: &str, source_path: Option<String>) -> CkadResult<Drawing> {
  let sections = split_sections(content);
  let part_name = sections
    .get(&100)
    .map(Vec::as_slice)
    .and_then(first_meaningful_line)
    .map(str::to_string);
  let (width, height) = sections
    .get(&200)
    .map(|lines| parse_sheet_extents(lines))
    .transpose()?
    .unwrap_or((None, None));

  let mut entities = Vec::new();
  if let Some(section) = sections.get(&300) {
    entities.extend(parse_geometry_section(section, "300")?);
  }

  let mut sheet = Sheet {
    index: Some(1),
    name: part_name,
    width,
    height,
    entities,
    bounds: None,
  };
  sheet.recompute_bounds();

  Ok(Drawing {
    source_path,
    sheets: vec![sheet],
    diagnostics: Vec::new(),
  })
}

fn split_sections(content: &str) -> BTreeMap<u32, Vec<String>> {
  let mut sections: BTreeMap<u32, Vec<String>> = BTreeMap::new();
  let mut current_id: Option<u32> = None;

  for line in content.lines() {
    let trimmed = line.trim();
    if trimmed.is_empty() {
      continue;
    }
    if let Some(id) = parse_section_header(trimmed) {
      current_id = Some(id);
      sections.entry(id).or_default();
      continue;
    }
    if let Some(id) = current_id {
      sections.entry(id).or_default().push(trimmed.to_string());
    }
  }

  sections
}

fn parse_section_header(line: &str) -> Option<u32> {
  let inner = line.strip_prefix('[')?.strip_suffix(']')?;
  inner.parse().ok()
}

fn first_meaningful_line(lines: &[String]) -> Option<&str> {
  lines
    .iter()
    .find(|line| !line.is_empty())
    .map(String::as_str)
}

fn parse_sheet_extents(lines: &[String]) -> CkadResult<(Option<f64>, Option<f64>)> {
  for line in lines {
    if let Some(rest) = line.strip_prefix("/E ") {
      let values = parse_floats(rest)?;
      if values.len() >= 4 {
        let width = values[2] - values[0];
        let height = values[3] - values[1];
        return Ok((Some(width), Some(height)));
      }
    }
  }
  Ok((None, None))
}

fn parse_geometry_section(lines: &[String], context: &str) -> CkadResult<Vec<Entity>> {
  let mut entities = Vec::new();
  let mut index = 0usize;
  while index < lines.len() {
    match lines[index].as_str() {
      "LINES" => {
        index += 1;
        let count = read_count(&lines, &mut index, context)?;
        for _ in 0..count {
          let coords = read_float_line(&lines, &mut index, context)?;
          skip_line(&lines, &mut index);
          if coords.len() >= 4 {
            entities.push(line_entity(
              coords[0],
              coords[1],
              coords[2],
              coords[3],
            ));
          }
        }
      }
      "POINTS" => {
        index += 1;
        let count = read_count(&lines, &mut index, context)?;
        index = index.saturating_add(count);
      }
      "CIRCLES" => {
        index += 1;
        let count = read_count(&lines, &mut index, context)?;
        for _ in 0..count {
          let coords = read_float_line(&lines, &mut index, context)?;
          if coords.len() >= 3 {
            entities.push(circle_entity(coords[0], coords[1], coords[2]));
          }
        }
      }
      "ARCS" => {
        index += 1;
        let count = read_count(&lines, &mut index, context)?;
        for _ in 0..count {
          let header = read_float_line(&lines, &mut index, context)?;
          skip_line(&lines, &mut index);
          let angles = read_float_line(&lines, &mut index, context)?;
          if header.len() >= 3 && angles.len() >= 2 {
            entities.push(arc_entity(
              header[0],
              header[1],
              header[2],
              angles[0],
              angles[1],
            ));
          }
        }
      }
      _ => index += 1,
    }
  }
  Ok(entities)
}

fn read_count(lines: &[String], index: &mut usize, context: &str) -> CkadResult<usize> {
  let line = lines
    .get(*index)
    .ok_or_else(|| CkadError::InvalidFormat {
      context: context.to_string(),
      message: "missing entity count".to_string(),
    })?;
  let count = line
    .trim()
    .parse::<usize>()
    .map_err(|err| CkadError::InvalidFormat {
      context: context.to_string(),
      message: format!("invalid entity count {line:?}: {err}"),
    })?;
  *index += 1;
  Ok(count)
}

fn read_float_line(lines: &[String], index: &mut usize, context: &str) -> CkadResult<Vec<f64>> {
  skip_extension_lines(lines, index);
  let line = lines
    .get(*index)
    .ok_or_else(|| CkadError::InvalidFormat {
      context: context.to_string(),
      message: "unexpected end of geometry section".to_string(),
    })?;
  if line.starts_with("OLE4DM") {
    return Err(CkadError::InvalidFormat {
      context: context.to_string(),
      message: format!("unexpected extension record {line:?}"),
    });
  }
  *index += 1;
  parse_floats(line)
}

fn skip_line(lines: &[String], index: &mut usize) {
  if *index < lines.len() {
    *index += 1;
  }
  skip_extension_lines(lines, index);
}

fn skip_extension_lines(lines: &[String], index: &mut usize) {
  while *index < lines.len() {
    let line = lines[*index].as_str();
    if line.starts_with("OLE4DM") {
      *index += 1;
      continue;
    }
    break;
  }
}

fn parse_floats(line: &str) -> CkadResult<Vec<f64>> {
  line
    .split_whitespace()
    .map(|token| {
      token.parse::<f64>().map_err(|err| CkadError::InvalidFormat {
        context: "numeric token".to_string(),
        message: format!("invalid float {token:?}: {err}"),
      })
    })
    .collect()
}

fn line_entity(x1: f64, y1: f64, x2: f64, y2: f64) -> Entity {
  Entity {
    layer: None,
    style: Style::default(),
    kind: EntityKind::Line {
      from: Point::new(x1, y1),
      to: Point::new(x2, y2),
    },
    provenance: None,
  }
}

fn circle_entity(cx: f64, cy: f64, radius: f64) -> Entity {
  Entity {
    layer: None,
    style: Style::default(),
    kind: EntityKind::Polyline(circle_polyline(cx, cy, radius)),
    provenance: None,
  }
}

fn arc_entity(cx: f64, cy: f64, radius: f64, start_deg: f64, end_deg: f64) -> Entity {
  Entity {
    layer: None,
    style: Style::default(),
    kind: EntityKind::Polyline(arc_polyline(
      cx,
      cy,
      radius,
      start_deg,
      end_deg,
    )),
    provenance: None,
  }
}

fn circle_polyline(cx: f64, cy: f64, radius: f64) -> Polyline {
  let points = arc_points(cx, cy, radius, 0.0, 360.0, CIRCLE_SEGMENTS);
  Polyline {
    points,
    closed: false,
  }
}

fn arc_polyline(cx: f64, cy: f64, radius: f64, start_deg: f64, end_deg: f64) -> Polyline {
  let points = arc_points(cx, cy, radius, start_deg, end_deg, ARC_SEGMENTS);
  Polyline {
    points,
    closed: false,
  }
}

fn arc_points(
  cx: f64,
  cy: f64,
  radius: f64,
  start_deg: f64,
  end_deg: f64,
  segments: usize,
) -> Vec<Point> {
  let (start, end) = normalize_arc_sweep(start_deg, end_deg);
  if segments == 0 {
    return Vec::new();
  }
  let step = (end - start) / segments as f64;
  (0..=segments)
    .map(|index| {
      let angle = start + step * index as f64;
      Point::new(cx + radius * angle.cos(), cy + radius * angle.sin())
    })
    .collect()
}

fn normalize_arc_sweep(start_deg: f64, end_deg: f64) -> (f64, f64) {
  let start = start_deg.to_radians();
  let mut end = end_deg.to_radians();
  if end <= start {
    end += std::f64::consts::TAU;
  }
  (start, end)
}

#[cfg(test)]
mod tests {
  use super::*;
  use dft2dxf_testkit::minimal_cnckad_dft;

  #[test]
  fn parses_minimal_cnckad_fixture() {
    let content = minimal_cnckad_dft();
    let drawing = parse_content(&content, None).unwrap();
    assert_eq!(drawing.sheets.len(), 1);
    assert_eq!(drawing.sheets[0].name.as_deref(), Some("TEST-PART"));
    assert_eq!(drawing.sheets[0].entities.len(), 1);
  }

  #[test]
  fn parses_utf16_le_cnckad_fixture() {
    let bytes = dft2dxf_testkit::minimal_cnckad_dft_utf16_le();
    let content = super::decode_text(&bytes);
    let drawing = parse_content(&content, None).unwrap();
    assert_eq!(drawing.sheets[0].entities.len(), 1);
  }
}
