# Roadmap

Incremental milestones for `dft2dxf`. Each milestone is independently reviewable.
Statuses refer to `docs/IMPLEMENTATION-STATUS.md`.

Synthetic fixtures prove internal plumbing; they **do not** prove Solid Edge compatibility.
M1 is mandatory before expanding conversion fidelity.

---

## M0 — Repository health and reproducible baseline

**Goal:** Truthful documentation, consistent line endings, deterministic negative fixtures,
and a recorded verification baseline.

**Scope:**

- `.gitattributes` for LF normalization
- Repair CRLF / bare-CR blockers for `rustfmt`
- Add missing malformed fixture bytes
- Remove misleading `build_line_emf` alias (tests use `build_rectangle_emf`)
- CI triggers on `main` and `master`
- Author `IMPLEMENTATION-STATUS.md` and this roadmap
- Honest README wording

**Non-goals:** New EMF/DXF/SVG features, API changes, dependency updates, real `.dft` files.

**Acceptance criteria:**

- `cargo fmt --all -- --check` passes
- `negative-sheet-count.bin` matches `negative_sheet_count_metadata()`
- No `build_line_emf` references remain
- Status doc distinguishes Verified / Implemented unverified / Planned / Blocked
- CI watches both `main` and `master`

**Fixtures required:** `tests/fixtures/malformed/negative-sheet-count.bin` (hand-crafted).

**Tests required:** Existing `metadata.rs::rejects_negative_sheet_count` (on-disk + synthetic).

**Likely files:** `.gitattributes`, `malformed_fixture.rs`, `emf_fixture.rs`, test imports,
`.github/workflows/ci.yml`, `docs/IMPLEMENTATION-STATUS.md`, `docs/ROADMAP.md`, `README.md`.

**Risks:** Large formatting-only diffs from `cargo fmt`.

**Why it cannot be skipped:** Without a honest baseline, later milestones cannot be judged;
fmt/CRLF failures block CI on Linux.

**Status:** Complete in working tree. `cargo fmt --check` verified; compile/test blocked locally by missing `link.exe`.

---

## M1 — Real DFT fixture acquisition and extraction validation

**Goal:** Prove the reader works on at least one legally distributable Solid Edge `.dft`
file independent of synthetic builders.

**Scope:**

- Acquire or author one redistributable `.dft` documented in `PROVENANCE.md`
- Smoke test: open → inspect → `extract_emf(1)` → valid EMF signature
- Fixture intake template and investigated-sources log
- Correct arc documentation (SVG omitted; DXF arc omitted with diagnostic)

**Non-goals:** Full conversion, EMF replay fidelity, golden SVG/DXF from real files, DXF `ARC` implementation.

**Acceptance criteria:**

- `tests/fixtures/valid/real-solid-edge.dft` committed with complete provenance **OR** honest blocker documented
- `opens_and_extracts_emf_from_real_solid_edge_dft_fixture` runs assertions when fixture present
- `INTAKE.md` describes remaining human steps
- No unsafe `Arc → CIRCLE` DXF mapping
- SVG arc not described as implemented

**Fixtures required:** ≥1 real `.dft` with signed-off provenance (canonical name: `real-solid-edge.dft`).

**Tests required:** `crates/dft-reader/tests/real_fixture.rs`; `omits_arc_entities_and_records_diagnostic` in `drawing-dxf`.

**Likely files:** `tests/fixtures/valid/`, `crates/dft-reader/tests/real_fixture.rs`, `drawing-dxf/src/writer.rs`, docs.

**Risks:** No public redistributable `.dft` found in automation environment; license ambiguity.

**Why it cannot be skipped:** Synthetic CFB files do not validate Solid Edge compatibility.

**Status:** **Partially completed / blocked.** Intake process, smoke-test scaffold, arc fixes, and docs are in place. **No real `.dft` committed.** Human must add `real-solid-edge.dft` per `INTAKE.md` to finish M1.

---

## M2 — Complete EMF header parsing and record-boundary validation

**Goal:** Parse `EMR_HEADER` fields (bounds, frame, `nBytes`, version) and validate record
stream integrity beyond signature checks.

**Scope:**

- Structured `EmfHeader` type in `emf-reader`
- Cross-check `nBytes` vs buffer length; bounds sanity
- Diagnostics for header/record anomalies

**Non-goals:** Graphics replay changes, DXF/SVG output changes.

**Acceptance criteria:**

- Unit tests for valid/invalid headers
- Malformed EMF tests extended
- Fuzz target continues without panics (run manually or in CI job)

**Fixtures required:** Synthetic EMF bytes; extracted EMF from M1 real fixture.

**Tests required:** `emf-reader` unit tests; optional header snapshot from real extract.

**Likely files:** `emf-reader/src/parser.rs`, new `header.rs`, `dft-reader/src/cfb.rs`.

**Risks:** Real EMF header variants may differ from synthetic 88-byte minimal header.

**Why it cannot be skipped:** Record boundaries depend on correct header semantics; silent
truncation causes deceptive partial output.

---

## M3 — Drawing IR stabilization and provenance

**Goal:** Stable IR types, consistent provenance on all emitted entities, diagnostic
schema suitable for CLI/JSON export.

**Scope:**

- Review `Entity` / `Style` / `Diagnostic` public API
- Ensure every replayed entity carries `SourceProvenance`
- Document IR JSON schema for tests

**Non-goals:** New geometry kinds beyond current set.

**Acceptance criteria:**

- Snapshot or serde round-trip tests for IR
- No breaking changes without version note in CHANGELOG (when introduced)

**Fixtures required:** Synthetic EMF → IR snapshots.

**Tests required:** `drawing-ir` unit tests; provenance assertions in `pipeline.rs`.

**Likely files:** `drawing-ir/src/*`, `emf-reader/src/replay.rs`.

**Risks:** API churn if done before M1/M2 stabilize inputs.

**Why it cannot be skipped:** Downstream SVG/DXF and diagnostics need a stable contract.

---

## M4 — SVG validation for core geometry

**Goal:** Golden SVG tests for each EMF-replayed primitive (rectangle, polyline, polygon,
moveto/lineto path, text) using deterministic EMF fixtures.

**Scope:**

- `build_polyline_emf` and similar **testkit-only** builders (not production replay expansion
  beyond existing code)
- Sheet-aware `viewBox` from IR sheet width/height
- Golden files under `tests/golden/svg/`

**Non-goals:** EMF pen/brush fidelity, clipping, raster embedding.

**Acceptance criteria:**

- One golden SVG per core record type
- Pipeline test compares normalized SVG string or insta snapshot
- Visual review optional; automated string compare required

**Fixtures required:** Synthetic EMF per record type; optionally real EMF extract from M1.

**Tests required:** `drawing-svg/tests/golden.rs` extensions; `emf-reader/tests/pipeline.rs`.

**Likely files:** `dft2dxf-testkit`, `drawing-svg`, golden SVG tree.

**Risks:** Coordinate system differences between EMF device units and sheet metadata.

**Why it cannot be skipped:** SVG is the primary validation surface per project positioning.

---

## M5 — EMF graphics-state replay: pens, selection, transforms

**Goal:** Replay `EMR_CREATEPEN`, `EMR_EXTCREATEPEN`, `EMR_SELECTOBJECT`, and basic map mode
so stroke color/width reflect EMF state.

**Scope:**

- Object table in replay state machine
- Map EMF colors/widths to IR `StrokeStyle`
- Diagnostics when object index is invalid

**Non-goals:** Brushes/fills, world transforms, clipping regions.

**Acceptance criteria:**

- Unit tests with synthetic EMF containing pen create + select + draw
- IR stroke differs from default black/1.0 in test

**Fixtures required:** New testkit EMF sequences; real drawing with varied line weights (M1).

**Tests required:** `emf-reader` replay tests; optional SVG golden.

**Likely files:** `emf-reader/src/replay.rs`, new `state.rs`, `record.rs`.

**Risks:** Object table indexing differences in Solid Edge EMF vs Win32 GDI reference.

**Why it cannot be skipped:** Most real drawings rely on pen selection for visible linework.

---

## M6 — DXF fidelity and golden DXF validation

**Goal:** Reliable DXF export with golden file regression tests and documented entity mapping.

**Scope:**

- Golden DXF for rectangle/polyline/text paths
- Layer/color passthrough where IR provides data (post-M5)
- CLI `convert` integration test

**Non-goals:** Full AutoCAD feature parity, blocks, dimensions as native entities.

**Acceptance criteria:**

- `tests/golden/dxf/*.dxf` with stable output
- `drawing-dxf/tests/writer.rs` compares normalized DXF text
- README fidelity section aligned with evidence

**Fixtures required:** IR or synthetic pipeline outputs.

**Tests required:** Golden DXF tests; CLI convert smoke test.

**Likely files:** `drawing-dxf`, `tests/golden/dxf/`, `dft2dxf-cli/tests/`.

**Risks:** `dxf` crate version quirks across platforms.

**Why it cannot be skipped:** DXF is a stated project output; without goldens it regresses silently.

---

## M7 — Text, clipping, fills, hatches, raster handling

**Goal:** Improve fidelity for non-line primitives; emit diagnostics where unsupported.

**Scope:**

- Correct `EMR_EXTTEXTOUT*` layout parsing against real EMF
- Clipping: diagnostic + optional IR clip path
- Fills/hatches/rasters: diagnostic at minimum; partial SVG/DXF where feasible

**Non-goals:** Perfect text metrics; embedded image recovery to external files.

**Acceptance criteria:**

- Fixture categories under `tests/fixtures/text/`, `hatches/`, etc. populated
- Each unsupported category produces explicit diagnostic with record offset

**Fixtures required:** Real or synthetic EMF per category (M1 corpus + targeted samples).

**Tests required:** Replay + SVG/DXF or diagnostic snapshot tests.

**Likely files:** `emf-reader`, `drawing-ir`, `drawing-svg`, `drawing-dxf`.

**Risks:** High format variability; patent/license on some compression schemes.

**Why it cannot be skipped:** Real drawings commonly include text and fill; silent omission
violates project policy.

---

## M8 — Fuzzing, CI hardening, release-quality CLI and API stabilization

**Goal:** Production-grade OSS infrastructure: fuzz in CI (or scheduled), MSRV policy,
documented public API, CLI JSON diagnostics, release process.

**Scope:**

- Add fuzz job or `cargo fuzz` documentation + smoke
- `convert --format json` diagnostics output
- `CHANGELOG.md`, crate README files, docs.rs metadata
- Semver policy for public crates

**Non-goals:** Cloud service, GUI, quotation automation.

**Acceptance criteria:**

- CI green on all platforms for full test matrix
- Security limits documented and tested
- v0.2.0 (or later) release candidate checklist complete

**Fixtures required:** Full corpus from M1+.

**Tests required:** Full workspace tests + fuzz smoke + deny/advisory clean.

**Likely files:** `.github/workflows/`, `dft2dxf-cli`, crate manifests, `SECURITY.md`.

**Risks:** Fuzz job runtime in CI; Windows MSVC dependency for contributors.

**Why it cannot be skipped:** Untrusted input handling requires sustained fuzzing and
release discipline.

---

## After M8

Future work (out of current roadmap): additional Solid Edge versions, performance tuning,
optional DXF layer export, community fixture corpus growth. Native CAD reconstruction
remains explicitly out of scope.
