//! Build minimal synthetic EMF files for tests.

/// EMF signature.
const EMF_SIGNATURE: u32 = 0x0000_464D;
/// `EMR_HEADER`
const EMR_HEADER: u32 = 1;
/// `EMR_EOF`
const EMR_EOF: u32 = 14;
/// `EMR_SETMAPMODE`
const EMR_SETMAPMODE: u32 = 17;
/// `EMR_MOVETOEX`
const EMR_MOVETOEX: u32 = 27;
/// `EMR_SELECTOBJECT`
const EMR_SELECTOBJECT: u32 = 37;
/// `EMR_CREATEPEN`
const EMR_CREATEPEN: u32 = 38;
/// `EMR_RECTANGLE`
const EMR_RECTANGLE: u32 = 42;
/// `EMR_ARC`
const EMR_ARC: u32 = 45;
/// `EMR_POLYLINE`
const EMR_POLYLINE: u32 = 4;
/// `EMR_LINETO`
const EMR_LINETO: u32 = 54;
/// `EMR_EXTTEXTOUTA`
const EMR_EXTTEXTOUTA: u32 = 83;

/// Builds a minimal valid EMF containing one rectangle record.
pub fn build_rectangle_emf(left: i32, top: i32, right: i32, bottom: i32) -> Vec<u8> {
  build_emf(&[rectangle_record(left, top, right, bottom)])
}

/// Builds an EMF with a polyline.
pub fn build_polyline_emf(points: &[(i32, i32)]) -> Vec<u8> {
  build_emf(&[polyline_record(points)])
}

/// Builds an EMF with one arc entity.
pub fn build_arc_emf(
  left: i32,
  top: i32,
  right: i32,
  bottom: i32,
  start_x: i32,
  start_y: i32,
  end_x: i32,
  end_y: i32,
) -> Vec<u8> {
  build_emf(&[arc_record(
    left, top, right, bottom, start_x, start_y, end_x, end_y,
  )])
}

/// Builds an EMF with pen creation, selection, move, and line.
pub fn build_pen_and_line_emf() -> Vec<u8> {
  let mut pen_payload = vec![0u8; 28];
  pen_payload[4..8].copy_from_slice(&1u32.to_le_bytes());
  pen_payload[12..16].copy_from_slice(&0x0000_FFu32.to_le_bytes());
  pen_payload[16..20].copy_from_slice(&2i32.to_le_bytes());

  let mut select_payload = vec![0u8; 12];
  select_payload[8..12].copy_from_slice(&1u32.to_le_bytes());

  let mut move_payload = vec![0u8; 16];
  move_payload[8..12].copy_from_slice(&0i32.to_le_bytes());
  move_payload[12..16].copy_from_slice(&0i32.to_le_bytes());

  let mut line_payload = vec![0u8; 16];
  line_payload[8..12].copy_from_slice(&100i32.to_le_bytes());
  line_payload[12..16].copy_from_slice(&50i32.to_le_bytes());

  build_emf(&[
    (EMR_CREATEPEN, pen_payload),
    (EMR_SELECTOBJECT, select_payload),
    (EMR_MOVETOEX, move_payload),
    (EMR_LINETO, line_payload),
  ])
}

/// Builds an EMF with ASCII text output.
pub fn build_text_emf(x: i32, y: i32, text: &str) -> Vec<u8> {
  let text_bytes = text.as_bytes();
  let payload_len = 24 + text_bytes.len() + 1;
  let padded_len = payload_len.next_multiple_of(4);
  let mut payload = vec![0u8; padded_len];
  payload[8..12].copy_from_slice(&x.to_le_bytes());
  payload[12..16].copy_from_slice(&y.to_le_bytes());
  payload[24..24 + text_bytes.len()].copy_from_slice(text_bytes);
  build_emf(&[(EMR_EXTTEXTOUTA, payload)])
}

/// Builds an EMF with mapping mode set to low metric (triggers scale).
pub fn build_transform_emf() -> Vec<u8> {
  let mut map_payload = vec![0u8; 12];
  map_payload[8..12].copy_from_slice(&8u32.to_le_bytes());
  build_emf(&[
    (EMR_SETMAPMODE, map_payload),
    rectangle_record(0, 0, 100, 50),
  ])
}

/// Builds an EMF with a polygon.
pub fn build_polygon_emf(points: &[(i32, i32)]) -> Vec<u8> {
  build_emf(&[polygon_record(points)])
}

/// Builds an EMF with an ellipse bounding box.
pub fn build_ellipse_emf(left: i32, top: i32, right: i32, bottom: i32) -> Vec<u8> {
  build_emf(&[ellipse_record(left, top, right, bottom)])
}

const EMR_POLYGON: u32 = 5;
const EMR_ELLIPSE: u32 = 49;

fn rectangle_record(left: i32, top: i32, right: i32, bottom: i32) -> (u32, Vec<u8>) {
  let mut payload = vec![0u8; 16];
  payload[0..4].copy_from_slice(&left.to_le_bytes());
  payload[4..8].copy_from_slice(&top.to_le_bytes());
  payload[8..12].copy_from_slice(&right.to_le_bytes());
  payload[12..16].copy_from_slice(&bottom.to_le_bytes());
  (EMR_RECTANGLE, payload)
}

fn polyline_record(points: &[(i32, i32)]) -> (u32, Vec<u8>) {
  let count = points.len() as u32;
  let mut payload = vec![0u8; 4 + points.len() * 8];
  payload[0..4].copy_from_slice(&count.to_le_bytes());
  let mut offset = 4usize;
  for (x, y) in points {
    payload[offset..offset + 4].copy_from_slice(&x.to_le_bytes());
    payload[offset + 4..offset + 8].copy_from_slice(&y.to_le_bytes());
    offset += 8;
  }
  (EMR_POLYLINE, payload)
}

fn polygon_record(points: &[(i32, i32)]) -> (u32, Vec<u8>) {
  let (_ty, payload) = polyline_record(points);
  (EMR_POLYGON, payload)
}

fn ellipse_record(left: i32, top: i32, right: i32, bottom: i32) -> (u32, Vec<u8>) {
  let mut payload = vec![0u8; 16];
  payload[0..4].copy_from_slice(&left.to_le_bytes());
  payload[4..8].copy_from_slice(&top.to_le_bytes());
  payload[8..12].copy_from_slice(&right.to_le_bytes());
  payload[12..16].copy_from_slice(&bottom.to_le_bytes());
  (EMR_ELLIPSE, payload)
}

fn arc_record(
  left: i32,
  top: i32,
  right: i32,
  bottom: i32,
  start_x: i32,
  start_y: i32,
  end_x: i32,
  end_y: i32,
) -> (u32, Vec<u8>) {
  let mut payload = vec![0u8; 32];
  payload[0..4].copy_from_slice(&left.to_le_bytes());
  payload[4..8].copy_from_slice(&top.to_le_bytes());
  payload[8..12].copy_from_slice(&right.to_le_bytes());
  payload[12..16].copy_from_slice(&bottom.to_le_bytes());
  payload[16..20].copy_from_slice(&start_x.to_le_bytes());
  payload[20..24].copy_from_slice(&start_y.to_le_bytes());
  payload[24..28].copy_from_slice(&end_x.to_le_bytes());
  payload[28..32].copy_from_slice(&end_y.to_le_bytes());
  (EMR_ARC, payload)
}

fn build_emf(records: &[(u32, Vec<u8>)]) -> Vec<u8> {
  let header_size = 88u32;
  let eof_size = 20u32;
  let body_size: u32 = records
    .iter()
    .map(|(_, payload)| 8 + payload.len() as u32)
    .sum();
  let file_size = header_size + body_size + eof_size;

  let mut data = Vec::new();
  append_record_header(&mut data, EMR_HEADER, header_size);
  data.extend_from_slice(&[0u8; 32]);
  data.extend_from_slice(&EMF_SIGNATURE.to_le_bytes());
  let remaining = header_size as usize - data.len();
  data.extend_from_slice(&vec![0u8; remaining]);
  if data.len() >= 52 {
    data[48..52].copy_from_slice(&file_size.to_le_bytes());
  }

  for (record_type, payload) in records {
    let size = 8 + payload.len() as u32;
    append_record_header(&mut data, *record_type, size);
    data.extend_from_slice(payload);
  }

  append_record_header(&mut data, EMR_EOF, eof_size);
  data.extend_from_slice(&vec![0u8; (eof_size as usize).saturating_sub(8)]);
  data
}

fn append_record_header(buf: &mut Vec<u8>, record_type: u32, size: u32) {
  buf.extend_from_slice(&record_type.to_le_bytes());
  buf.extend_from_slice(&size.to_le_bytes());
}

/// Validates that bytes contain a valid EMF signature in the header record.
pub fn is_emf(data: &[u8]) -> bool {
  data.len() >= 44
    && u32::from_le_bytes(data[0..4].try_into().unwrap()) == EMR_HEADER
    && u32::from_le_bytes(data[40..44].try_into().unwrap()) == EMF_SIGNATURE
}
