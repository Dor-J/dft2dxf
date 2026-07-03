# Architecture

## Workspace crates

| Crate | Responsibility |
| --- | --- |
| `ckad-reader` | cncKad text `.dft` → Drawing IR |
| `dft-reader` | CFB inspection, draft metadata parsing, bounded EMF extraction |
| `emf-reader` | EMF record parsing and graphics-state replay |
| `drawing-ir` | Canonical vector drawing model and diagnostics |
| `drawing-svg` | Deterministic SVG output for validation |
| `drawing-dxf` | DXF writer (ARC/CIRCLE, layers, CAM layers) |
| `dft2dxf-cli` | CLI commands and structured JSON output |
| `dft2dxf-testkit` | Synthetic fixtures and test helpers |
| `dft2dxf-core` | In-memory convert / inspect / validate API |
| `dft2dxf-sidecar` | Axum HTTP worker for backend integration |

## Data flow

```text
cncKad .dft (text)
  -> ckad-reader
  -> drawing-ir
  -> drawing-svg / drawing-dxf

Solid Edge .dft (CFB)
  -> dft-reader
  -> embedded zlib EMF per sheet
  -> emf-reader
  -> drawing-ir
  -> drawing-svg / drawing-dxf
```

## Backend integration

| Pattern | Description |
| --- | --- |
| CLI subprocess | FastAPI spawns `dft2dxf convert` per job — **ready** |
| HTTP sidecar | `dft2dxf-sidecar` calls `dft2dxf-core` in-process — **ready** |

See [backend-integration.md](backend-integration.md).

## Design boundary

`dft2dxf` converts the visible vector representation embedded in draft viewer
EMF streams (Solid Edge) or native cncKad text sections. It is not a native Solid Edge CAD parser.
