# Limitations

## Solid Edge (EMF path)

- **Ceiling:** EMF exposes **visual primitives only** (lines, curves, text, pen color/width).
  There are **no layers, semantic dimensions, CAM, material/thickness, or parametric objects**
  in embedded viewer streams.
- No recovery of native Solid Edge drawing objects, constraints, or associative views
- No guarantee that hidden CAD objects appear in EMF
- EMF coverage is incremental; unsupported record types are logged as diagnostics
- Real-world DFT layout variation across Solid Edge versions is still being validated
- **Real fixture blocker:** no redistributable Solid Edge `.dft` is committed; SE fidelity
  is validated on synthetic EMF fixtures only until `tests/fixtures/valid/real-solid-edge.dft`
  is added per [INTAKE.md](../tests/fixtures/valid/INTAKE.md)

## cncKad (text path)

- Layer/color metadata uses **heuristic column mapping**; verify against your parts in CAD
- CAM operation geometry export is **best-effort** (lossless raw lines in `--cam-json`)
- Dimension entities in IR are supported but cncKad dimension sections are not parsed yet
- `OLE4DM` extension lines are skipped

## Output writers

- DXF fidelity depends on target viewer (AutoCAD, LibreCAD, etc.)
- SVG uses computed bounds with Y-flip; extremely large coordinates may need padding tuning
- Text rotation/font fidelity varies by source (EMF vs cncKad)

## General

- DXF/SVG are export views, not editable source CAD
- CLI diagnostics for batch conversion are summary-level; use `--format json` for machine output
