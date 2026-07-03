# DXF Mapping

## Entity mappings

| Drawing IR | DXF entity | Status |
| --- | --- | --- |
| `Line` | `LINE` | supported |
| `Polyline` | `LWPOLYLINE` | supported |
| `Rectangle` | closed `LWPOLYLINE` | supported |
| `Path` | `LWPOLYLINE` | supported |
| `Text` | `TEXT` | supported |
| `Arc` | `ARC` | supported |
| `Circle` | `CIRCLE` | supported |
| `Dimension` | `LINE` / `TEXT` fallback | partial |
| CAM operations | `PUNCH` / `CUT` / `TOOLS` layers | supported (cncKad) |

## Unsupported / intentionally omitted

| Drawing IR | Behavior |
| --- | --- |
| EMF fills / hatches | Diagnostic `emf.fill_unsupported`; not exported |
| EMF clipping | Diagnostic `emf.clipping_unsupported` |
| EMF raster | Diagnostic `emf.raster_unsupported` |

## Known fidelity loss

- EMF font metrics do not map cleanly to DXF text height/width
- Fills, hatches, gradients, and transparency are not preserved
- Solid Edge EMF has no layer semantics in IR
- DXF ACI color mapping is heuristic for non-primary RGB values

DXF output is a visual/interoperability export, not editable source CAD.

## Golden tests

Regression fixtures live under `tests/golden/dxf/`. Tests normalize volatile header
fields (`$HANDSEED`, GUIDs) before comparison.
