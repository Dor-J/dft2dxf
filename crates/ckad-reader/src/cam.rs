//! cncKad CAM section parser (`[1100]` / `[1200]`).

use drawing_ir::{CamOperation, CamProgram, CamTool, Point};

use crate::error::{CkadError, CkadResult};

/// Parses the tool table section `[1100]`.
pub fn parse_tool_section(lines: &[String]) -> CkadResult<Vec<CamTool>> {
  let mut tools = Vec::new();
  let mut index = 0usize;
  while index < lines.len() {
    let line = lines[index].trim();
    if line.is_empty() {
      index += 1;
      continue;
    }
    if line.starts_with('R') || line.starts_with('C') {
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() >= 2 {
        let kind = parts[0].to_string();
        let size = parts[1]
          .parse::<f64>()
          .map_err(|err| parse_err("tool size", line, &err.to_string()))?;
        let size2 = parts.get(2).and_then(|value| value.parse::<f64>().ok());
        let mut comment = None;
        index += 1;
        while index < lines.len() {
          let next = lines[index].trim();
          if next.starts_with('R') || next.starts_with('C') {
            break;
          }
          if let Some(text) = next.strip_prefix("TOOLCM") {
            comment = Some(parse_quoted_comment(text));
          }
          index += 1;
        }
        tools.push(CamTool {
          kind,
          size,
          size2,
          comment,
        });
        continue;
      }
    }
    index += 1;
  }
  Ok(tools)
}

/// Parses the operations section `[1200]`.
pub fn parse_operations_section(lines: &[String]) -> CkadResult<Vec<CamOperation>> {
  let mut operations = Vec::new();
  let mut index = 0usize;
  while index < lines.len() {
    let line = lines[index].trim();
    if line.is_empty() {
      index += 1;
      continue;
    }
    match line {
      "ONLINE" | "SINGLE" | "ONARC" => {
        let kind = line.to_string();
        index += 1;
        let mut raw = vec![kind.clone()];
        while index < lines.len() {
          let next = lines[index].trim();
          if next.is_empty() {
            index += 1;
            continue;
          }
          if next == "ONLINE" || next == "SINGLE" || next == "ONARC" {
            break;
          }
          raw.push(next.to_string());
          index += 1;
        }
        operations.push(map_operation(&kind, raw)?);
      }
      _ => index += 1,
    }
  }
  Ok(operations)
}

fn map_operation(kind: &str, raw: Vec<String>) -> CkadResult<CamOperation> {
  match kind {
    "ONLINE" => Ok(CamOperation::Online { raw }),
    "ONARC" => Ok(CamOperation::OnArc { raw }),
    "SINGLE" => {
      let position = raw.iter().find_map(|line| parse_single_position(line));
      let tool_index = raw.iter().find_map(|line| parse_single_tool_index(line));
      Ok(CamOperation::Single {
        position,
        tool_index,
        raw,
      })
    }
    other => Err(CkadError::InvalidFormat {
      context: "cam operation".to_string(),
      message: format!("unknown operation kind {other:?}"),
    }),
  }
}

fn parse_single_position(line: &str) -> Option<Point> {
  let values = parse_floats(line);
  if values.len() >= 6 {
    let x = values[4];
    let y = values[5];
    if x.is_finite() && y.is_finite() {
      return Some(Point::new(x, y));
    }
  }
  None
}

fn parse_single_tool_index(line: &str) -> Option<u32> {
  let values = parse_floats(line);
  values.get(3).and_then(|value| f64_to_u32(*value))
}

fn parse_quoted_comment(text: &str) -> String {
  text
    .trim()
    .trim_start_matches('"')
    .trim_end_matches('"')
    .trim()
    .to_string()
}

fn parse_floats(line: &str) -> Vec<f64> {
  line
    .split_whitespace()
    .filter_map(|token| token.parse::<f64>().ok())
    .collect()
}

fn parse_err(context: &str, line: &str, message: &str) -> CkadError {
  CkadError::InvalidFormat {
    context: context.to_string(),
    message: format!("invalid token in {line:?}: {message}"),
  }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn f64_to_u32(value: f64) -> Option<u32> {
  if value.is_finite() && value.fract() == 0.0 && value > 0.0 && value <= f64::from(u32::MAX) {
    Some(value as u32)
  } else {
    None
  }
}

/// Parses CAM sections into a [`CamProgram`].
///
/// # Errors
///
/// Returns [`CkadError`] if either CAM section contains invalid numeric data.
pub fn parse_cam(
  tools: Option<&[String]>,
  operations: Option<&[String]>,
) -> CkadResult<CamProgram> {
  let tools = tools
    .map(parse_tool_section)
    .transpose()?
    .unwrap_or_default();
  let operations = operations
    .map(parse_operations_section)
    .transpose()?
    .unwrap_or_default();
  Ok(CamProgram { tools, operations })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_tool_table() {
    let lines = vec![
      "4 5".to_string(),
      "R 50 5".to_string(),
      "1 0 0".to_string(),
      "TOOLCM \"(M)\"".to_string(),
      "C 10".to_string(),
      "1 0 0".to_string(),
      "TOOLCM \" punch\"".to_string(),
    ];
    let tools = parse_tool_section(&lines).unwrap();
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].kind, "R");
    assert_eq!(tools[1].kind, "C");
  }

  #[test]
  fn parses_operations_section() {
    use dft2dxf_testkit::professional_cnckad_dft;
    let content = professional_cnckad_dft();
    let ops_start = content.find("[1200]").unwrap();
    let ops_end = content.find("OLE4DM").unwrap();
    let block: Vec<String> = content[ops_start..ops_end]
      .lines()
      .skip(1)
      .map(str::to_string)
      .collect();
    let operations = parse_operations_section(&block).unwrap();
    assert!(!operations.is_empty());
    let tools_start = content.find("[1100]").unwrap();
    let tools_end = content.find("[1200]").unwrap();
    let tool_lines: Vec<String> = content[tools_start..tools_end]
      .lines()
      .skip(1)
      .filter(|line| !line.trim().is_empty() && !line.starts_with('['))
      .map(str::to_string)
      .collect();
    let program = parse_cam(Some(&tool_lines), Some(&block)).unwrap();
    assert!(program.tools.len() >= 2);
    assert_eq!(program.operations.len(), operations.len());
  }
}
