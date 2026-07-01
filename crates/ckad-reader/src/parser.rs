//! cncKad text `.dft` geometry parser.

use std::collections::BTreeMap;
use std::path::Path;

use drawing_ir::{ArcSegment, Drawing, Entity, EntityKind, Point, Sheet, Style};

use crate::cam::parse_cam;
use crate::error::{CkadError, CkadResult};
use crate::metadata::{
  parse_kfactor_section, parse_part_section, parse_sheet_section, parse_thickness_sections,
};
use crate::style::{inline_color, inline_layer_id, EntityMeta};

/// Default maximum input file size (50 MiB).
pub const DEFAULT_MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Reads a cncKad text `.dft` file into [`Drawing`] IR.
///
/// # Errors
///
/// Returns [`CkadError`] if the file cannot be read, exceeds `max_file_size`, is not recognized
/// as cncKad text, or contains invalid drawing data.
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
///
/// # Errors
///
/// Returns [`CkadError`] if any recognized section contains invalid numeric or geometry data.
pub fn parse_content(content: &str, source_path: Option<String>) -> CkadResult<Drawing> {
  let sections = split_sections(content);
  let (part_name, customer) = sections
    .get(&100)
    .map_or((None, None), |lines| parse_part_section(lines));
  let (width, height, mut metadata) = sections
    .get(&200)
    .map(|lines| parse_sheet_section(lines))
    .transpose()?
    .unwrap_or((None, None, drawing_ir::DrawingMetadata::default()));
  metadata.part_name.clone_from(&part_name);
  metadata.customer = customer;
  if let Some(k_factor) = sections.get(&210).map(|lines| parse_kfactor_section(lines)) {
    metadata.k_factor = k_factor;
  }
  let thickness_sections: Vec<(u32, Vec<String>)> = sections
    .iter()
    .filter(|(id, _)| (500..=503).contains(*id))
    .map(|(id, lines)| (*id, lines.clone()))
    .collect();
  if let Some(thickness) = parse_thickness_sections(&thickness_sections) {
    metadata.thickness = Some(thickness);
  }

  let mut entities = Vec::new();
  if let Some(section) = sections.get(&300) {
    entities.extend(parse_geometry_section(section, "300")?);
  }
  if let Some(section) = sections.get(&310) {
    entities.extend(parse_geometry_section(section, "310")?);
  }

  let cam = parse_cam(
    sections.get(&1100).map(Vec::as_slice),
    sections.get(&1200).map(Vec::as_slice),
  )?;
  let cam = if cam.tools.is_empty() && cam.operations.is_empty() {
    None
  } else {
    Some(cam)
  };

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
    metadata,
    cam,
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

fn parse_geometry_section(lines: &[String], context: &str) -> CkadResult<Vec<Entity>> {
  let mut entities = Vec::new();
  let mut index = 0usize;
  while index < lines.len() {
    match lines[index].as_str() {
      "LINES" => {
        index += 1;
        let count = read_count(lines, &mut index, context)?;
        for _ in 0..count {
          let coords = read_float_line(lines, &mut index, context)?;
          let meta = read_metadata_line(lines, &mut index);
          if coords.len() >= 4 {
            entities.push(line_entity(
              coords[0], coords[1], coords[2], coords[3], meta,
            ));
          }
        }
      }
      "POINTS" => {
        index += 1;
        let count = read_count(lines, &mut index, context)?;
        index = index.saturating_add(count);
      }
      "CIRCLES" => {
        index += 1;
        let count = read_count(lines, &mut index, context)?;
        for _ in 0..count {
          let coords = read_float_line(lines, &mut index, context)?;
          if coords.len() >= 3 {
            let meta = entity_meta_from_inline(&coords);
            entities.push(circle_entity(coords[0], coords[1], coords[2], meta));
          }
        }
      }
      "ARCS" => {
        index += 1;
        let count = read_count(lines, &mut index, context)?;
        for _ in 0..count {
          let header = read_float_line(lines, &mut index, context)?;
          if header.len() < 3 {
            continue;
          }
          let meta = entity_meta_from_inline(&header);
          let (start_deg, end_deg) = read_arc_angles(lines, &mut index, context)?;
          entities.push(arc_entity(
            header[0], header[1], header[2], start_deg, end_deg, meta,
          ));
        }
      }
      _ => index += 1,
    }
  }
  Ok(entities)
}

fn read_arc_angles(lines: &[String], index: &mut usize, context: &str) -> CkadResult<(f64, f64)> {
  skip_extension_lines(lines, index);
  let next = read_float_line(lines, index, context)?;
  if next.len() >= 4 {
    let angles = read_float_line(lines, index, context)?;
    if angles.len() >= 2 {
      return Ok((angles[0], angles[1]));
    }
    return Err(CkadError::InvalidFormat {
      context: context.to_string(),
      message: "arc missing angle line".to_string(),
    });
  }
  if next.len() >= 2 {
    return Ok((next[0], next[1]));
  }
  Err(CkadError::InvalidFormat {
    context: context.to_string(),
    message: "arc missing angle data".to_string(),
  })
}

fn read_count(lines: &[String], index: &mut usize, context: &str) -> CkadResult<usize> {
  let line = lines.get(*index).ok_or_else(|| CkadError::InvalidFormat {
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
  let line = lines.get(*index).ok_or_else(|| CkadError::InvalidFormat {
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

fn read_metadata_line(lines: &[String], index: &mut usize) -> EntityMeta {
  skip_extension_lines(lines, index);
  if *index >= lines.len() {
    return EntityMeta::default();
  }
  let line = &lines[*index];
  if line.starts_with("OLE4DM") {
    return EntityMeta::default();
  }
  if line.split_whitespace().count() >= 4 {
    let meta = EntityMeta::from_line(line);
    *index += 1;
    return meta;
  }
  EntityMeta::default()
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
      token
        .parse::<f64>()
        .map_err(|err| CkadError::InvalidFormat {
          context: "numeric token".to_string(),
          message: format!("invalid float {token:?}: {err}"),
        })
    })
    .collect()
}

fn entity_meta_from_inline(values: &[f64]) -> EntityMeta {
  EntityMeta {
    layer_id: inline_layer_id(values),
    color_aci: inline_color(values),
  }
}

fn apply_meta(entity: &mut Entity, meta: EntityMeta) {
  entity.layer = meta.layer_name();
  if meta.color_aci.is_some() {
    entity.style = meta.style();
  }
}

fn line_entity(x1: f64, y1: f64, x2: f64, y2: f64, meta: EntityMeta) -> Entity {
  let mut entity = Entity {
    layer: None,
    style: Style::default(),
    kind: EntityKind::Line {
      from: Point::new(x1, y1),
      to: Point::new(x2, y2),
    },
    provenance: None,
  };
  apply_meta(&mut entity, meta);
  entity
}

fn circle_entity(cx: f64, cy: f64, radius: f64, meta: EntityMeta) -> Entity {
  let mut entity = Entity {
    layer: None,
    style: Style::default(),
    kind: EntityKind::Circle {
      center: Point::new(cx, cy),
      radius,
    },
    provenance: None,
  };
  apply_meta(&mut entity, meta);
  entity
}

fn arc_entity(
  cx: f64,
  cy: f64,
  radius: f64,
  start_deg: f64,
  end_deg: f64,
  meta: EntityMeta,
) -> Entity {
  let (start, end) = normalize_arc_sweep(start_deg, end_deg);
  let mut entity = Entity {
    layer: None,
    style: Style::default(),
    kind: EntityKind::Arc(ArcSegment {
      center: Point::new(cx, cy),
      radius,
      start_angle: start,
      end_angle: end,
    }),
    provenance: None,
  };
  apply_meta(&mut entity, meta);
  entity
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

  #[test]
  fn parses_professional_cnckad_fixture() {
    let content = dft2dxf_testkit::professional_cnckad_dft();
    let drawing = parse_content(&content, None).unwrap();
    assert!(drawing.metadata.thickness.is_some());
    assert!(drawing.metadata.k_factor.is_some());
    assert!(drawing.cam.is_some());
    assert!(drawing.sheets[0]
      .entities
      .iter()
      .any(|entity| matches!(entity.kind, EntityKind::Circle { .. })));
    assert!(drawing.sheets[0]
      .entities
      .iter()
      .any(|entity| matches!(entity.kind, EntityKind::Arc(_))));
  }
}
