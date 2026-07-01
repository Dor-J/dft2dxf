# DXF Mapping

## Current experimental mappings

| Drawing IR | DXF entity | Status |
| --- | --- | --- |
| `Line` | `LINE` | supported |
| `Polyline` | `LWPOLYLINE` | supported |
| `Rectangle` | closed `LWPOLYLINE` | supported |
| `Path` | `LWPOLYLINE` | supported |
| `Text` | `TEXT` | supported |
| `Arc` | — | **unsupported** (omitted; diagnostic `dxf.unsupported_entity`) |

## Unsupported / intentionally omitted

| Drawing IR | Behavior |
| --- | --- |
| `Arc` | Not exported. A prior `CIRCLE` substitution was removed because mapping a partial arc to a full circle changes geometry. Proper DXF `ARC` output is planned for a later milestone. |

## Known fidelity loss

- EMF font metrics do not map cleanly to DXF text height/width
- Fills, hatches, gradients, and transparency are not preserved
- Clipping and complex transforms may be lost
- Arc entities are omitted from DXF until `ARC` export exists

DXF output should be treated as a visual/interoperability export, not an editable
Solid Edge replacement.
