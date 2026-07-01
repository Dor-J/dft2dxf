# Implementation Status

Last updated: 2026-07-01 (M0 baseline audit).

## Executive Summary

`dft2dxf` is an early-stage Rust workspace with the intended crate boundaries in place.
A **synthetic** DFT → EMF extraction → EMF record parse → Drawing IR → SVG/DXF path is
implemented in source and covered by integration tests, but **could not be executed on the
current Windows development host** because the MSVC linker (`link.exe`) is missing.

**Verified locally (M0):** `cargo fmt --all -- --check` passes; CRLF/bare-CR blockers in
`malformed_fixture.rs` are repaired; missing `negative-sheet-count.bin` fixture is present;
misleading `build_line_emf` test helper removed; CI now triggers on `main` and `master`.

**Not verified locally:** `cargo check`, `cargo test`, `cargo clippy`, `cargo deny` (see
[Build and CI Status](#build-and-ci-status)).

**Real Solid Edge `.dft` compatibility is unproven.** All committed DFT/EMF coverage is
synthetic (`dft2dxf-testkit`) or hand-crafted malformed metadata bytes. No `.dft` files are
committed.

**Safe on untrusted input:** Limits and structured errors are implemented in library code.
Fuzz targets exist under `fuzz/` but are not run in CI. Control/state EMF records are
silently skipped during replay; only unrecognized record types produce IR diagnostics.

## Verification Environment

| Item | Value |
| --- | --- |
| Date | 2026-07-01 |
| OS | Windows 10 (build 26200) |
| Shell | PowerShell |
| Rust | `cargo 1.88.0` (`%USERPROFILE%\.cargo\bin\cargo.exe`) |
| Toolchain pin | `rust-toolchain.toml` → 1.88 with rustfmt, clippy |
| Git branch | `master` (`git branch --show-current`) |
| Git remote | `origin` → `https://github.com/Dor-J/dft2dxf.git` |
| HEAD | `50d8d92` — *Enforce safe Rust policy across the project* |

Commands run in this environment are listed under [Build and CI Status](#build-and-ci-status).

## Current Architecture

| Crate | Responsibility | Key paths |
| --- | --- | --- |
| `dft-reader` | CFB open, storage walk, `JDraftDocumentInfo` parse, bounded zlib decompress, EMF header validation, per-sheet extraction | `crates/dft-reader/src/` |
| `emf-reader` | EMF record iteration and graphics replay into Drawing IR | `crates/emf-reader/src/parser.rs`, `replay.rs` |
| `drawing-ir` | Canonical vector model, diagnostics, provenance | `crates/drawing-ir/src/` |
| `drawing-svg` | Deterministic SVG serialization | `crates/drawing-svg/src/writer.rs` |
| `drawing-dxf` | Experimental DXF writer via `dxf` crate | `crates/drawing-dxf/src/writer.rs` |
| `dft2dxf-cli` | `inspect`, `extract-emf`, `validate`, `convert` | `crates/dft2dxf-cli/src/main.rs` |
| `dft2dxf-testkit` | Synthetic DFT/EMF builders and malformed metadata bytes | `crates/dft2dxf-testkit/src/` |
| `fuzz/` (workspace excluded) | libFuzzer targets for metadata, zlib, EMF records | `fuzz/fuzz_targets/` |

**Data flow:** `.dft` (CFB) → `dft-reader` → zlib EMF per sheet → `emf-reader` →
`drawing-ir` → `drawing-svg` / `drawing-dxf`.

## Capability Matrix

| Capability | Status | Evidence |
| --- | --- | --- |
| Open CFB `.dft` | **Implemented, unverified** | `dft-reader/src/storage.rs::open_compound_file`; test `opens_synthetic_dft_and_extracts_emf` |
| Enumerate storage and streams | **Implemented, unverified** | `build_storage_tree`, `walk_storage`; CLI test `inspect_command_lists_synthetic_sheet` |
| Locate `JDraftViewerInfo` | **Implemented, unverified** | `metadata.rs` constants; `StorageTree.has_viewer_info` |
| Parse `JDraftDocumentInfo` / sheet metadata | **Implemented, unverified** | `parse_draft_metadata`; tests in `metadata.rs`, `multi_sheet.rs` |
| Numbered per-sheet streams (`JDraftViewerInfo/1`, …) | **Implemented, unverified** | `extract_sheet_emf`; `multi_sheet.rs` |
| Bounded zlib decompress | **Implemented, unverified** | `decompress.rs`; `decompression.rs` |
| Extract EMF bytes / write `.emf` | **Implemented, unverified** | `ExtractedEmf::write_to`; CLI `extract_emf_writes_file` |
| EMF header validation at extract | **Implemented, unverified** | `cfb.rs::validate_emf_header` (type, size, signature) |
| EMF record boundary parsing | **Implemented, unverified** | `EmfDocument::parse`; `malformed.rs` negative tests |
| EMF replay to Drawing IR | **Implemented, unverified** | `replay_to_drawing`; `pipeline.rs` (rectangle synthetic path) |
| SVG output | **Implemented, unverified** | `drawing-svg`; golden tests `svg_line_matches_golden_file`, `svg_renders_path_segments` |
| DXF output | **Implemented, unverified** | `drawing-dxf`; `writer.rs` tests assert `SECTION`, `LWPOLYLINE` |
| CLI `inspect` / `extract-emf` / `validate` | **Implemented, unverified** | `crates/dft2dxf-cli/tests/cli.rs` |
| CLI `convert` | **Implemented, unverified** | `main.rs::cmd_convert`; **no dedicated CLI test** |
| Real Solid Edge `.dft` open/extract | **Planned / missing** | No `.dft` fixtures committed (`glob **/*.dft` → 0 files) |
| Graphics-state replay (pens, brushes, fonts) | **Planned / missing** | `CREATEPEN`/`SELECTOBJECT` constants only; replay uses default stroke |
| Fuzzing in CI | **Planned / missing** | `fuzz/` exists; not referenced in `.github/workflows/ci.yml` |
| `cargo fmt --check` | **Verified** | Exit 0 on 2026-07-01 after M0 hygiene fixes |
| Full workspace compile + test | **Blocked** | `link.exe` not found (MSVC Build Tools not installed) |

## DFT Container and EMF Extraction Status

**Supported layout (synthetic only):**

- Storage `JDraftViewerInfo`
- Stream `JDraftViewerInfo/JDraftDocumentInfo` — sheet names, dimensions, EMF size fields
- Per-sheet streams `JDraftViewerInfo/{1..N}` — zlib-compressed EMF payloads

**Implemented behaviors:**

- File size, stream size, decompressed size, sheet count, CFB depth/entry limits
  (`dft-reader/src/limits.rs`)
- EMF signature check after decompress (`validate_emf_header`)
- Declared `emf_size` mismatch tolerated (comment in `extract_sheet_emf`)

**Unknown / unvalidated against real Solid Edge output:**

- Alternate compression wrappers, storage layouts, or viewer format versions
- Multi-sheet real drawings, background sheets, linked geometry

## EMF Replay Support Matrix

Legend: **Replay** = converted to Drawing IR entities; **Diagnostic** = `Drawing.diagnostics`
entry; **Silent** = parsed but ignored in `replay.rs` `_ => {}` branch.

| Record / feature | Replay | Tested E2E | SVG | DXF | Notes |
| --- | --- | --- | --- | --- | --- |
| `EMR_HEADER` | Silent (control) | Partial | — | — | Validated at DFT extract, not fully parsed in `emf-reader` |
| `EMR_EOF` | Silent (control) | Yes | — | — | Required by `EmfDocument::parse` |
| `EMR_SETMAPMODE` | Silent (control) | No | — | — | |
| `EMR_SELECTOBJECT` | Silent (control) | No | — | — | Object table not replayed |
| `EMR_CREATEPEN` / `EMR_EXTCREATEPEN` | Silent (control) | No | — | — | Default stroke used |
| `EMR_RECTANGLE` | Replay | **Yes** (synthetic) | Unverified | Unverified | Primary tested record type |
| `EMR_POLYLINE` / `EMR_POLYLINE16` | Replay | No | No | No | Code in `replay.rs` only |
| `EMR_POLYGON` / `EMR_POLYGON16` | Replay | No | No | No | Mapped to closed polyline |
| `EMR_MOVETOEX` / `EMR_LINETO` | Replay | No | No | No | Path coalescing heuristic |
| `EMR_EXTTEXTOUTA` / `EMR_EXTTEXTOUTW` | Replay | No | No | No | Simplified text offset parsing |
| Unrecognized record types | Diagnostic | Weak | — | — | `reports_unsupported_records_in_replay` does not assert diagnostics |
| Arcs, ellipses, beziers, fills, clips, rasters | Planned / missing | No | No | No | |

## SVG Output Status

| Feature | Status | Evidence |
| --- | --- | --- |
| Line, polyline, path, rectangle, text entities | **Implemented, unverified** | `drawing-svg/src/writer.rs::render_entity` |
| Arc entities | **Implemented, unverified** | Renders empty group (`EntityKind::Arc`) |
| Sheet-sized viewBox | **Planned / missing** | Hard-coded `viewBox="0 0 1000 1000"` |
| EMF-derived colors/widths | **Planned / missing** | Uses IR stroke or black default |
| Golden SVG (IR-level) | **Implemented, unverified** | `tests/golden/svg/line.svg`, `line.snap`, `path.snap` |
| Golden SVG from EMF replay | **Planned / missing** | Pipeline test only checks `svg` substring |

## DXF Output Status

| Feature | Status | Evidence |
| --- | --- | --- |
| `LINE` | **Implemented, unverified** | `map_entity` |
| `LWPOLYLINE` (polyline, rectangle, path) | **Implemented, unverified** | `writer.rs`; tests check `LWPOLYLINE` |
| `TEXT` | **Implemented, unverified** | `map_entity` |
| `CIRCLE` for arc IR | **Implemented, unverified** | Approximation noted in `docs/dxf-mapping.md` |
| Layers, colors, lineweights | **Planned / missing** | Not mapped from EMF |
| Fills, hatches, images, clipping | **Planned / missing** | |
| Golden DXF | **Planned / missing** | `tests/golden/dxf/.gitkeep` only |
| CLI surfaces conversion diagnostics | **Planned / missing** | `cmd_convert` does not print `Drawing.diagnostics` |

DXF entities emitted are **real geometry** for supported IR kinds, not placeholder stubs.
Visual fidelity against Solid Edge or CAD viewers is **not validated**.

## Test and Fixture Coverage

### Synthetic fixture coverage

| Area | Tests | Builder |
| --- | --- | --- |
| DFT open / inspect / extract | `extraction.rs`, `multi_sheet.rs`, CLI `cli.rs` | `build_minimal_dft`, `build_rectangle_emf` |
| Metadata negatives | `metadata.rs` | `malformed_fixture.rs` + on-disk `.bin` |
| Zlib limits | `decompression.rs` | `invalid_zlib_payload`, synthetic DFT |
| EMF parse / replay / pipeline | `pipeline.rs`, `malformed.rs` | `build_rectangle_emf` |
| SVG golden (IR) | `golden.rs` | Hand-built `Drawing` |
| DXF writer | `writer.rs` | Synthetic DFT pipeline |

### Real Solid Edge `.dft` fixture coverage

**None committed.** `tests/fixtures/valid/` contains only `.gitkeep` and `PROVENANCE.md`.

### On-disk malformed fixtures

| File | Status |
| --- | --- |
| `tests/fixtures/malformed/too-short-metadata.bin` | Present |
| `tests/fixtures/malformed/negative-sheet-count.bin` | Added in M0 (18 bytes, matches `negative_sheet_count_metadata()`) |

### Fixture categories with placeholders only

`edge-cases/`, `multi-sheet/`, `text/`, `dimensions/`, `hatches/`, `raster-content/`,
`transforms/` — `.gitkeep` only.

## Security and Parser Hardening Status

| Control | Status | Location |
| --- | --- | --- |
| `max_file_size` (256 MiB) | **Implemented, unverified** | `limits.rs`, `open_compound_file` |
| `max_stream_size` (64 MiB) | **Implemented, unverified** | `read_stream_limited`, decompress input |
| `max_decompressed_size` (256 MiB) | **Implemented, unverified** | `decompress.rs` |
| `max_sheet_count`, CFB depth/entries | **Implemented, unverified** | `limits.rs`, `walk_storage`, metadata parse |
| EMF `max_record_count` / `max_record_size` | **Implemented, unverified** | `emf-reader/src/parser.rs` |
| Checked arithmetic on offsets | **Implemented, unverified** | `ByteCursor`, `EmfDocument::parse` |
| Polyline point-count cap in replay | **Planned / missing** | `parse_poly` uses `Vec::with_capacity(count)` without upper bound vs limits |
| Fuzz targets | **Implemented, unverified** | `fuzz/fuzz_targets/*.rs`; not in CI |
| Safe Rust policy script | **Implemented, unverified** | `scripts/check-safe-rust.sh` (CI step) |

## Build and CI Status

| Command | Result | Exit | Notes |
| --- | --- | --- | --- |
| `git status --short` | Clean aside from M0 edits | — | Run after M0 file changes |
| `git branch --show-current` | `master` | 0 | |
| `git remote -v` | `origin` → `https://github.com/Dor-J/dft2dxf.git` | 0 | |
| `git log --oneline -20` | 12 commits | 0 | Initial import through safe-Rust policy |
| `cargo fmt --all -- --check` | **PASS** | 0 | Verified 2026-07-01 |
| `cargo check --workspace --all-targets` | **FAIL** | 101 | `linker 'link.exe' not found` |
| `cargo test --workspace --all-targets` | **Blocked** | — | Same linker error |
| `cargo clippy --workspace --all-targets -- -D warnings` | **Blocked** | — | Same linker error |
| `cargo deny check` | **Blocked** | 101 | `cargo-deny` not installed locally; CI installs it on Ubuntu |

CI workflow (`.github/workflows/ci.yml`): safe-Rust script, fmt, clippy, test on
ubuntu/windows/macos; `cargo deny` on ubuntu. Triggers on **`main` and `master`** (M0).

## Known Limitations

- No native Solid Edge CAD semantics (constraints, parameters, associative views).
- EMF coverage is incremental; most record types lack end-to-end tests.
- Synthetic CFB layout does not prove compatibility with production Solid Edge files.
- Diagnostics for unsupported EMF records are stored in Drawing IR but not exposed by CLI.
- SVG viewBox and stroke styling are not sheet-accurate.
- Windows local builds require MSVC `link.exe` (documented in README).

## Technical Debt, Ranked

1. **No real `.dft` corpus** — highest risk to project usefulness (M1).
2. **Local/Windows compile blocked without MSVC** — contributor friction.
3. **EMF replay tests cover rectangle path only** — overstates readiness.
4. **Control EMF records silently skipped** — can produce silently incomplete output.
5. **CLI `convert` untested and does not emit diagnostics**.
6. **Golden DXF missing** — DXF regressions undetected.
7. **Fuzz targets not in CI**.
8. **Branch naming** — `master` locally vs historical `main` in workflow (mitigated by dual trigger).

## Recommended Next Milestone

**M1 — Real DFT fixture acquisition and extraction validation** (see `docs/ROADMAP.md`).

Obtain or create one legally distributable Solid Edge `.dft` file, prove standalone EMF
sheet extraction, and add a smoke test plus provenance documentation. Do not expand EMF
replay or DXF scope until M1 passes in CI.
