# Implementation Status

Last updated: 2026-07-01 (M1 pass — real fixture blocked; arc doc/DXF fixes applied).

## Executive Summary

`dft2dxf` is an early-stage Rust workspace with the intended crate boundaries in place.
A **synthetic** DFT → EMF extraction → EMF record parse → Drawing IR → SVG/DXF path is
implemented in source and covered by integration tests, but **could not be executed on the
current Windows development host** because the MSVC linker (`link.exe`) is missing.

**Verified locally (M0 + M1 hygiene):** `cargo fmt --all -- --check` passes.

**M1 status: partially completed / blocked.** A real-fixture smoke test and intake process
exist, but **no legally redistributable Solid Edge `.dft` file was committed**. Real DFT
extraction remains **Planned / missing** until `tests/fixtures/valid/real-solid-edge.dft`
is added per [INTAKE.md](../tests/fixtures/valid/INTAKE.md).

**Arc output truthfulness (M1 pre-work):** SVG `EntityKind::Arc` is **intentionally omitted**
(empty group). DXF `Arc` entities are **omitted** with diagnostic `dxf.unsupported_entity`
(unsafe `Arc → CIRCLE` mapping removed).

**Not verified locally:** `cargo check`, `cargo test`, `cargo clippy`, `cargo deny`.

**Safe on untrusted input:** Limits and structured errors are implemented in library code.
Fuzz targets exist under `fuzz/` but are not run in CI.

## Verification Environment

| Item | Value |
| --- | --- |
| Date | 2026-07-01 |
| OS | Windows 10 (build 26200) |
| Shell | PowerShell |
| Rust | `cargo 1.88.0` (`%USERPROFILE%\.cargo\bin\cargo.exe`) |
| Toolchain pin | `rust-toolchain.toml` → 1.88 with rustfmt, clippy |
| Git branch | `master` |
| Git remote | `origin` → `https://github.com/Dor-J/dft2dxf.git` |

## Current Architecture

| Crate | Responsibility | Key paths |
| --- | --- | --- |
| `dft-reader` | CFB open, storage walk, metadata parse, bounded zlib decompress, EMF extraction | `crates/dft-reader/src/` |
| `emf-reader` | EMF record iteration and graphics replay into Drawing IR | `crates/emf-reader/src/` |
| `drawing-ir` | Canonical vector model, diagnostics, provenance | `crates/drawing-ir/src/` |
| `drawing-svg` | Deterministic SVG serialization | `crates/drawing-svg/src/writer.rs` |
| `drawing-dxf` | Experimental DXF writer | `crates/drawing-dxf/src/writer.rs` |
| `dft2dxf-cli` | CLI commands | `crates/dft2dxf-cli/src/main.rs` |
| `dft2dxf-testkit` | Synthetic DFT/EMF builders | `crates/dft2dxf-testkit/src/` |

## Capability Matrix

| Capability | Status | Evidence |
| --- | --- | --- |
| Open CFB `.dft` (synthetic) | **Implemented, unverified** | `extraction.rs`, `dft2dxf-testkit` |
| Real Solid Edge `.dft` open/extract | **Planned / missing** | No `real-solid-edge.dft`; smoke test skips (`real_fixture.rs`) |
| Real-fixture smoke test scaffold | **Implemented, unverified** | `opens_and_extracts_emf_from_real_solid_edge_dft_fixture` |
| Fixture intake process | **Verified** | `tests/fixtures/valid/INTAKE.md`, `PROVENANCE.md` |
| EMF replay (rectangle path, synthetic) | **Implemented, unverified** | `pipeline.rs` |
| SVG line/polyline/path/rectangle/text | **Implemented, unverified** | `drawing-svg/src/writer.rs`; golden tests |
| SVG arc | **Unsupported / intentionally omitted** | `EntityKind::Arc` → empty group (`writer.rs`) |
| DXF line/polyline/rectangle/path/text | **Implemented, unverified** | `drawing-dxf` tests |
| DXF arc | **Unsupported / diagnostic-only** | Omitted; `dxf.unsupported_entity` (`writer.rs`, `omits_arc_entities_and_records_diagnostic`) |
| `cargo fmt --check` | **Verified** | Exit 0 (2026-07-01) |
| Full workspace compile + test | **Blocked** | `link.exe` not found |

## DFT Container and EMF Extraction Status

**Validated layout (synthetic only):** `JDraftViewerInfo` / `JDraftDocumentInfo` / numbered sheet streams.

**Real Solid Edge layout:** Unknown until M1 fixture committed and inspected.

## EMF Replay Support Matrix

(See prior matrix; unchanged in M1 — no replay scope expansion.)

## SVG Output Status

| Feature | Status | Evidence |
| --- | --- | --- |
| Line, polyline, path, rectangle, text | **Implemented, unverified** | `render_entity` |
| Arc | **Unsupported / intentionally omitted** | Empty `<g>`; no arc path geometry |
| Sheet-sized viewBox | **Planned / missing** | Fixed `0 0 1000 1000` |
| Golden SVG (IR-level) | **Implemented, unverified** | `tests/golden/svg/` |

## DXF Output Status

| Feature | Status | Evidence |
| --- | --- | --- |
| `LINE`, `LWPOLYLINE`, `TEXT` | **Implemented, unverified** | `map_entity` |
| `Arc` → `CIRCLE` | **Removed (was unsafe)** | Replaced by omission + diagnostic |
| `Arc` → `ARC` | **Planned / missing** | Future milestone |
| Golden DXF | **Planned / missing** | `tests/golden/dxf/.gitkeep` |

Supported IR kinds emit real geometry. Arc entities are **not** exported.

## Test and Fixture Coverage

### Synthetic

Unchanged from M0 — `dft2dxf-testkit` drives DFT/EMF integration tests.

### Real Solid Edge `.dft`

| Item | Status |
| --- | --- |
| Committed fixture | **Absent** (`real-solid-edge.dft` expected) |
| Smoke test | `crates/dft-reader/tests/real_fixture.rs` (skips if absent) |
| Intake docs | `INTAKE.md`, `PROVENANCE.md` |
| SHA-256 on record | Pending fixture |

## Real Solid Edge Fixture Validation

- **Fixture filename:** `real-solid-edge.dft` (not committed)
- **Source:** Pending — see [INTAKE.md](../tests/fixtures/valid/INTAKE.md)
- **Redistribution status:** Not established
- **Known Solid Edge version:** Unknown
- **SHA-256:** N/A
- **Expected sheet count:** TBD
- **Expected extracted EMF sheet count:** ≥ 1 (smoke test uses sheet 1)
- **Extracted EMF validation method:** `extract_emf` → `validate_emf_header` + `is_emf()`
- **Known limitations:** M1 is extraction-only; no SVG/DXF fidelity claims from extraction

### Independent EMF validation

| Field | Value |
| --- | --- |
| Validation tool | Not run |
| Reason | No real extracted EMF artifact available |
| Automated minimum | `validate_emf_header` in `dft-reader` extract path |

## Security and Parser Hardening Status

Unchanged from M0 (limits enforced in code; fuzz not in CI).

## Build and CI Status

| Command | Result | Exit | Notes |
| --- | --- | --- | --- |
| `cargo fmt --all -- --check` | **PASS** | 0 | 2026-07-01 |
| `cargo check --workspace --all-targets` | **FAIL** | 101 | `link.exe` not found |
| `cargo test --workspace --all-targets` | **Blocked** | — | Same linker error |
| `cargo clippy --workspace --all-targets -- -D warnings` | **Blocked** | — | Same linker error |
| `cargo deny check` | **Blocked** | 101 | `cargo-deny` not installed locally |

## Known Limitations

- No real `.dft` corpus committed (M1 blocker).
- SVG/DXF arc output not implemented.
- Synthetic tests do not prove Solid Edge compatibility.
- Windows builds need MSVC `link.exe`.

## Technical Debt, Ranked

1. **Commit redistributable real `.dft`** (completes M1).
2. Local compile blocked without MSVC.
3. EMF replay tested only for rectangle synthetic path.
4. Control EMF records silently skipped in replay.
5. Golden DXF missing.

## Recommended Next Milestone

Complete **M1** by committing `real-solid-edge.dft` with full provenance, then re-run CI.
After M1 is **Verified**, proceed to **M2** (EMF header parsing) per `docs/ROADMAP.md`.
