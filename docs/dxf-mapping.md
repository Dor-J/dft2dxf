# DXF Mapping

## Current experimental mappings

| Drawing IR | DXF entity |
| --- | --- |
| `Line` | `LINE` |
| `Polyline` | `LWPOLYLINE` |
| `Rectangle` | closed `LWPOLYLINE` |
| `Path` | `LWPOLYLINE` |
| `Text` | `TEXT` |
| `Arc` | `CIRCLE` (approximation) |

## Known fidelity loss

- EMF font metrics do not map cleanly to DXF text height/width
- Fills, hatches, gradients, and transparency are not preserved
- Clipping and complex transforms may be lost
- Arcs may be approximated when exact DXF arc parameters are unavailable

DXF output should be treated as a visual/interoperability export, not an editable
Solid Edge replacement.
