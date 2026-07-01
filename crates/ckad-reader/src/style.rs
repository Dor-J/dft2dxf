//! cncKad AutoCAD Color Index (ACI) helpers.

use drawing_ir::{Color, StrokeStyle, Style};

/// Maps an ACI index to an approximate RGB color.
#[must_use]
pub fn aci_to_rgb(aci: u8) -> Color {
  match aci {
    1 => Color::rgb(255, 0, 0),
    2 => Color::rgb(255, 255, 0),
    3 => Color::rgb(0, 255, 0),
    4 => Color::rgb(0, 255, 255),
    5 => Color::rgb(0, 0, 255),
    6 => Color::rgb(255, 0, 255),
    7 => Color::rgb(255, 255, 255),
    8 => Color::rgb(128, 128, 128),
    9 => Color::rgb(192, 192, 192),
    12 => Color::rgb(0, 128, 255),
    15 => Color::rgb(255, 128, 0),
    _ => Color::black(),
  }
}

/// Builds a stroke style from an ACI color index.
#[must_use]
pub fn style_from_aci(aci: u8) -> Style {
  Style {
    stroke: Some(StrokeStyle {
      color: aci_to_rgb(aci),
      width: 1.0,
    }),
    fill: None,
  }
}

/// Parses trailing metadata columns on geometry records.
#[derive(Debug, Clone, Copy, Default)]
pub struct EntityMeta {
  /// Layer identifier from the metadata line.
  pub layer_id: Option<i32>,
  /// ACI color index when present.
  pub color_aci: Option<u8>,
}

impl EntityMeta {
  /// Parses a metadata line such as `1 0 269 0 15 0`.
  #[must_use]
  pub fn from_line(line: &str) -> Self {
    let values = line
      .split_whitespace()
      .filter_map(|token| token.parse::<f64>().ok())
      .collect::<Vec<_>>();
    let layer_id = values.get(2).and_then(|value| {
      let id = *value as i32;
      if id > 0 {
        Some(id)
      } else {
        None
      }
    });
    let color_aci = values.get(4).and_then(|value| {
      let aci = *value as u8;
      if (1..=255).contains(&aci) {
        Some(aci)
      } else {
        None
      }
    });
    Self {
      layer_id,
      color_aci,
    }
  }

  /// Returns a layer name for DXF/SVG export.
  #[must_use]
  pub fn layer_name(&self) -> Option<String> {
    self.layer_id.map(|id| format!("L{id}"))
  }

  /// Returns a style from metadata when color is known.
  #[must_use]
  pub fn style(&self) -> Style {
    self.color_aci.map(style_from_aci).unwrap_or_default()
  }
}

/// Parses inline color from extended geometry lines (`cx cy r color ...`).
#[must_use]
pub fn inline_color(values: &[f64]) -> Option<u8> {
  values.get(3).and_then(|value| {
    let aci = *value as u8;
    if (1..=255).contains(&aci) {
      Some(aci)
    } else {
      None
    }
  })
}

/// Parses inline layer id from extended geometry lines.
#[must_use]
pub fn inline_layer_id(values: &[f64]) -> Option<i32> {
  values.get(5).and_then(|value| {
    let id = *value as i32;
    if id > 0 {
      Some(id)
    } else {
      None
    }
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn aci_to_rgb_maps_red() {
    let color = aci_to_rgb(1);
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 0);
  }

  #[test]
  fn entity_meta_from_line_parses_layer_and_color() {
    let meta = EntityMeta::from_line("1 0 269 0 15 0");
    assert_eq!(meta.layer_id, Some(269));
    assert_eq!(meta.color_aci, Some(15));
    assert_eq!(meta.layer_name(), Some("L269".to_string()));
  }

  #[test]
  fn inline_color_and_layer_id() {
    let values = [100.0, 50.0, 10.0, 15.0, 0.0, 269.0];
    assert_eq!(inline_color(&values), Some(15));
    assert_eq!(inline_layer_id(&values), Some(269));
  }
}
