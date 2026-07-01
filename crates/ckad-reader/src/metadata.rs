//! cncKad metadata section parser.

use drawing_ir::{DrawingMetadata, PaperUnit};

use crate::error::{CkadError, CkadResult};

/// Parses section `[100]` into part/customer fields.
#[must_use]
pub fn parse_part_section(lines: &[String]) -> (Option<String>, Option<String>) {
  let mut values = lines
    .iter()
    .map(|line| line.trim())
    .filter(|line| !line.is_empty());
  let part_name = values.next().map(str::to_string);
  let customer = values.next().map(str::to_string);
  (part_name, customer)
}

/// Parses section `[200]` for sheet extents, scale, and material hints.
pub fn parse_sheet_section(
  lines: &[String],
) -> CkadResult<(Option<f64>, Option<f64>, DrawingMetadata)> {
  let mut metadata = DrawingMetadata {
    units: PaperUnit::Millimeters,
    ..Default::default()
  };
  let mut width = None;
  let mut height = None;

  for line in lines {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("/E ") {
      let values = parse_floats(rest)?;
      if values.len() >= 4 {
        width = Some(values[2] - values[0]);
        height = Some(values[3] - values[1]);
      }
    } else if let Some(rest) = trimmed.strip_prefix("/P ") {
      let values = parse_floats(rest)?;
      if values.len() >= 3 {
        metadata.scale = Some(values[2]);
      }
    } else if let Some(rest) = trimmed.strip_prefix("/M ") {
      let values = parse_floats(rest)?;
      if let Some(material_id) = values.first().and_then(|value| f64_to_i32(*value)) {
        metadata.material = Some(format!("M{material_id}"));
      }
    }
  }

  Ok((width, height, metadata))
}

/// Parses section `[210]` for K-factor.
#[must_use]
pub fn parse_kfactor_section(lines: &[String]) -> Option<f64> {
  for line in lines {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("KFactor") {
      return rest.trim().parse().ok();
    }
    if let Ok(value) = trimmed.parse::<f64>() {
      return Some(value);
    }
  }
  None
}

/// Parses thickness from sections `[500]`..`[503]`.
#[must_use]
pub fn parse_thickness_sections(sections: &[(u32, Vec<String>)]) -> Option<f64> {
  for (id, lines) in sections {
    if (500..=503).contains(id) {
      for line in lines {
        if let Ok(value) = line.trim().parse::<f64>() {
          if value > 0.0 {
            return Some(value);
          }
        }
      }
    }
  }
  None
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

#[allow(clippy::cast_possible_truncation)]
fn f64_to_i32(value: f64) -> Option<i32> {
  if value.is_finite()
    && value.fract() == 0.0
    && value >= f64::from(i32::MIN)
    && value <= f64::from(i32::MAX)
  {
    Some(value as i32)
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_part_section_extracts_names() {
    let lines = vec!["PART-A".to_string(), "CUSTOMER-B".to_string()];
    let (part, customer) = parse_part_section(&lines);
    assert_eq!(part.as_deref(), Some("PART-A"));
    assert_eq!(customer.as_deref(), Some("CUSTOMER-B"));
  }

  #[test]
  fn parse_sheet_section_extents_and_scale() {
    let lines = vec![
      "/E 0 0 200 100".to_string(),
      "/P 200 100 1000 25".to_string(),
      "/M 4 0 1".to_string(),
    ];
    let (width, height, meta) = parse_sheet_section(&lines).unwrap();
    assert_eq!(width, Some(200.0));
    assert_eq!(height, Some(100.0));
    assert_eq!(meta.scale, Some(1000.0));
    assert_eq!(meta.material.as_deref(), Some("M4"));
  }

  #[test]
  fn parse_kfactor_section_reads_value() {
    assert_eq!(
      super::parse_kfactor_section(&["KFactor 0.400000".to_string()]),
      Some(0.4)
    );
  }

  #[test]
  fn parse_thickness_from_section_503() {
    let sections = vec![(503, vec!["1.500000".to_string()])];
    assert_eq!(parse_thickness_sections(&sections), Some(1.5));
  }
}
