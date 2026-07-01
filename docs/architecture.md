# Architecture

## Workspace crates

| Crate | Responsibility |
| --- | --- |
| `dft-reader` | CFB inspection, draft metadata parsing, bounded EMF extraction |
| `emf-reader` | EMF record parsing and graphics-state replay |
| `drawing-ir` | Canonical vector drawing model and diagnostics |
| `drawing-svg` | Deterministic SVG output for validation |
| `drawing-dxf` | Experimental DXF writer |
| `dft2dxf-cli` | CLI commands and structured output |
| `dft2dxf-testkit` | Synthetic fixtures and test helpers |

## Data flow

```text
.dft (CFB)
  -> dft-reader
  -> embedded zlib EMF per sheet
  -> emf-reader
  -> drawing-ir
  -> drawing-svg / drawing-dxf
```

## Design boundary

`dft2dxf` converts the visible vector representation embedded in draft viewer
EMF streams. It is not a native Solid Edge CAD parser.
