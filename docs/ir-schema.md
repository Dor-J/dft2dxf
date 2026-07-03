# Drawing IR JSON Schema

The `drawing-ir` crate serializes to JSON for CLI `--format json`, CAM sidecars, and the HTTP
sidecar API. All public types implement `Serialize` and `Deserialize`.

## Conventions

| Rule | Detail |
| --- | --- |
| Field names | `snake_case` |
| Enum tags | `EntityKind` uses `"kind"`; `DimensionKind` uses `"type"`; `CamOperation` uses `"kind"` |
| Omitted fields | `Option::None` and empty collections use `skip_serializing_if` |
| Coordinates | `f64` in drawing units (typically millimeters for cncKad) |

## Top-level `Drawing`

```json
{
  "sheets": [ { "index": 1, "name": "Sheet1", "entities": [] } ],
  "metadata": { "units": "millimeters", "part_name": "PART-001" },
  "cam": { "tools": [], "operations": [] },
  "diagnostics": []
}
```

## `Entity` / `EntityKind`

Tagged by `"kind"`:

- `line` — `{ "from": { "x", "y" }, "to": { "x", "y" } }`
- `polyline` — `{ "points": [...], "closed": bool }`
- `path` — `{ "segments": [ { "kind": "move_to" | "line_to" | "close", ... } ] }`
- `rectangle` — `{ "top_left", "bottom_right" }`
- `arc` — `{ "center", "radius", "start_angle", "end_angle" }`
- `circle` — `{ "center", "radius" }`
- `dimension` — nested `DimensionKind`
- `text` — `TextRun` object

Each entity may include `layer`, `style`, and `provenance` (`emf_record_index`, `emf_record_type`).

## `Diagnostic`

```json
{
  "severity": "warning",
  "code": "emf.unsupported_record",
  "message": "unsupported EMF record type 0x00000099",
  "provenance": { "emf_record_index": 3, "emf_record_type": 153 }
}
```

## Stability

Breaking JSON shape changes require a semver minor/major bump and an entry in `CHANGELOG.md`.
Round-trip tests in `crates/drawing-ir/tests/serde_roundtrip.rs` guard compatibility.
