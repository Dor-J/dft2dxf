# Implementation Status

Last updated: 2026-07-01 (professional conversion milestone).

## Executive Summary

`dft2dxf` converts **cncKad text `.dft`** files to professional-grade DXF/SVG with geometry,
layers, metadata, and CAM. **Solid Edge** compound `.dft` files are supported via embedded EMF
replay with a documented visual fidelity ceiling.

**cncKad path:** verified against gitignored local fixtures (`validate-fixtures --local`).

**Solid Edge path:** synthetic EMF tests pass; **no redistributable real `.dft` committed**
(see [INTAKE.md](../tests/fixtures/valid/INTAKE.md)).

## Architecture

| Crate | Responsibility |
| --- | --- |
| `ckad-reader` | cncKad text parser (geometry, metadata, CAM) |
| `dft-reader` | Solid Edge CFB + EMF extraction |
| `emf-reader` | EMF replay (lines, arcs, pens, transforms, text) |
| `drawing-ir` | Canonical model + metadata + `CamProgram` |
| `drawing-dxf` | Native ARC/CIRCLE, layers, units, CAM layers |
| `drawing-svg` | Computed viewBox, layer groups, Y-flip |
| `dft2dxf-cli` | `convert`, `convert-all`, `--cam-json`, diagnostics |

## Capability Matrix

| Capability | cncKad | Solid Edge EMF |
| --- | --- | --- |
| Lines / polylines | Yes | Yes |
| Native circles | Yes | Yes (ellipse) |
| Native arcs | Yes | Yes (partial) |
| Layers / colors | Yes (heuristic) | No |
| Part / customer | Yes | No |
| Material / thickness / K-factor | Yes | No |
| CAM tools + operations | Yes (+ JSON sidecar) | No |
| Text | Planned | Basic |
| Dimensions | IR only | No |

## DXF Writer

| Feature | Status |
| --- | --- |
| `LINE`, `LWPOLYLINE`, `TEXT` | Implemented |
| Native `ARC`, `CIRCLE` | Implemented |
| Layer table + ACI colors | Implemented |
| `$INSUNITS` / `$MEASUREMENT` / extents | Implemented |
| CAM layers (`PUNCH`, `CUT`, `TOOLS`) | Implemented |

## SVG Writer

| Feature | Status |
| --- | --- |
| Computed viewBox from bounds | Implemented |
| Layer groups (`data-layer`) | Implemented |
| Arc / circle rendering | Implemented |
| Y-flip for CAD coordinates | Implemented |

## Solid Edge Fixture Status

| Item | Status |
| --- | --- |
| `real-solid-edge.dft` | **Absent** (blocker documented in INTAKE.md) |
| Smoke test | Skips when fixture missing |
| EMF fidelity | Synthetic + expanded record replay |

## Documentation

- [cnckad-format.md](cnckad-format.md) — section-by-section cncKad decode notes
- [limitations.md](limitations.md) — honest ceilings per source format

## Verification Commands

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings

# Line coverage (>= 80% gate; excludes dft2dxf-testkit)
bash scripts/coverage.sh
# Windows: cargo llvm-cov --workspace --exclude dft2dxf-testkit --summary-only --fail-under-lines 80
```

## Testing

| Metric | Status |
| --- | --- |
| Workspace tests | ~100+ integration + unit tests |
| Line coverage (`cargo llvm-cov`) | **≥ 80%** enforced in CI (Ubuntu) |
| CI fixtures | `tests/fixtures/valid/ci/` (cncKad + synthetic Solid Edge) |
| Local cncKad smoke | `validate-fixtures --local` (gitignored fixtures) |

```bash
dft2dxf validate-fixtures --local
dft2dxf convert-all --local --dxf-dir target/out/dxf --svg-dir target/out/svg --cam-json
```
