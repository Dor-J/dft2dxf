# Implementation Status

Last updated: 2026-07-03 (M2–M9 milestone completion).

## Executive Summary

`dft2dxf` converts **cncKad text `.dft`** files to professional-grade DXF/SVG with geometry,
layers, metadata, and CAM. **Solid Edge** compound `.dft` files are supported via embedded EMF
replay with a documented visual fidelity ceiling.

| Path | Verification | Production note |
| --- | --- | --- |
| **cncKad** | Local fixtures (`validate-fixtures --local`) + CI synthetic fixtures | **Ready** for backend integration via CLI subprocess |
| **Solid Edge** | Synthetic EMF tests only; no redistributable real `.dft` | Use for preview/SVG; expect EMF fidelity ceiling |

| **cncKad** | **Ready** for subprocess or sidecar integration |

## Roadmap Progress

Statuses: **Complete** · **Partial** · **Planned** · **Blocked**

| Milestone | Goal | Status | Remaining work |
| --- | --- | --- | --- |
| **M0** | Repository health, fmt/CI baseline | **Complete** | — |
| **M1** | Real Solid Edge `.dft` fixture + smoke test | **Blocked** | Commit `real-solid-edge.dft` with provenance per [INTAKE.md](../tests/fixtures/valid/INTAKE.md) |
| **M2** | Structured `EmfHeader` + record-boundary validation | **Complete** | — |
| **M3** | Drawing IR stabilization + provenance | **Complete** | — |
| **M4** | Golden SVG per EMF-replayed primitive | **Complete** | — |
| **M5** | EMF pens, selection, transforms | **Complete** | — |
| **M6** | Golden DXF regression tests | **Complete** | Optional committed `rectangle.dxf` golden file |
| **M7** | Text, clipping, fills, hatches, raster | **Complete** | Synthetic category tests; real EMF corpus still M1-blocked |
| **M8** | Fuzzing, release-quality CI/API | **Complete** | crates.io publish optional |
| **M9** | Backend integration (CLI + HTTP sidecar) | **Complete** | — |

See [ROADMAP.md](ROADMAP.md) for full acceptance criteria per milestone.

## Architecture

| Crate | Responsibility |
| --- | --- |
| `ckad-reader` | cncKad text parser (geometry, metadata, CAM) |
| `dft-reader` | Solid Edge CFB + EMF extraction |
| `emf-reader` | EMF record parsing and graphics-state replay |
| `drawing-ir` | Canonical model + metadata + `CamProgram` (serde JSON) |
| `drawing-dxf` | Native ARC/CIRCLE, layers, units, CAM layers |
| `drawing-svg` | Computed viewBox, layer groups, Y-flip |
| `dft2dxf-cli` | `convert`, `convert-all`, `--cam-json`, `--format json` |
| `dft2dxf-sidecar` | Axum HTTP worker (`/health`, `/v1/convert`, …) |

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
| EMF pens / transforms | N/A | Partial (replay code present) |

## DXF Writer

| Feature | Status |
| --- | --- |
| `LINE`, `LWPOLYLINE`, `TEXT` | Implemented |
| Native `ARC`, `CIRCLE` | Implemented |
| Layer table + ACI colors | Implemented |
| `$INSUNITS` / `$MEASUREMENT` / extents | Implemented |
| CAM layers (`PUNCH`, `CUT`, `TOOLS`) | Implemented |
| Golden file regression (`tests/golden/dxf/`) | Planned (M6) |

## SVG Writer

| Feature | Status |
| --- | --- |
| Computed viewBox from bounds | Implemented |
| Layer groups (`data-layer`) | Implemented |
| Arc / circle rendering | Implemented |
| Y-flip for CAD coordinates | Implemented |
| Golden snapshots (line, path) | Partial (M4) |

## CI & Quality Gates

| Check | Status |
| --- | --- |
| `cargo fmt --all -- --check` | **Passing** (Ubuntu, Windows, macOS) |
| `cargo clippy --all-targets -- -D warnings` | **Passing** (3 OS) |
| `cargo test --all` | **Passing** (3 OS) |
| Line coverage ≥ 80% (`cargo llvm-cov`) | **Passing** (Ubuntu) |
| `cargo deny check` | **Passing** (Ubuntu) |
| Safe Rust policy (`scripts/check-safe-rust.sh`) | **Enforced** |
| Fuzz targets in CI | **Passing** (Ubuntu fuzz-smoke job) |

Rust toolchain: **1.88** (see `rust-toolchain.toml`).

## Solid Edge Fixture Status

| Item | Status |
| --- | --- |
| `real-solid-edge.dft` | **Absent** (blocker documented in INTAKE.md) |
| Smoke test | Skips when fixture missing |
| EMF fidelity | Synthetic + expanded record replay |

## Backend Integration

| Pattern | Status | Documentation |
| --- | --- | --- |
| CLI subprocess from FastAPI | **Ready** | [backend-integration.md](backend-integration.md) |
| HTTP sidecar (Axum worker pool) | **Ready** | `dft2dxf-sidecar` + [deploy/docker-compose.yml](../deploy/docker-compose.yml) |

**Ready today:** build `dft2dxf` release binary, call `convert` from a FastAPI background worker,
return DXF/SVG/CAM JSON from temp files.

**Not yet available:** in-process Python bindings (PyO3), published pip package, or the Axum sidecar crate.

## Documentation

- [backend-integration.md](backend-integration.md) — FastAPI subprocess + planned Axum sidecar
- [cnckad-format.md](cnckad-format.md) — section-by-section cncKad decode notes
- [limitations.md](limitations.md) — honest ceilings per source format
- [ROADMAP.md](ROADMAP.md) — milestone acceptance criteria

## Verification Commands

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# Line coverage (>= 80% gate; excludes dft2dxf-testkit)
bash scripts/coverage.sh
# Windows: cargo llvm-cov --workspace --exclude dft2dxf-testkit --summary-only --fail-under-lines 80

cargo deny check
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
