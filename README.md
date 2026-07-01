# dft2dxf

Convert `.dft` draft files into portable **DXF** and **SVG** outputs.

The tool auto-detects the input format and routes to the appropriate parser:

| Format | Detection | Data source |
| --- | --- | --- |
| **Metalix cncKad** | Text header `gKad` / `CKad` (ASCII or UTF-16 LE) | Native text sections — geometry, layers, metadata, CAM |
| **Solid Edge** | OLE compound file (`D0 CF 11 E0 …`) | Embedded viewer EMF streams |

cncKad is the **primary, full-fidelity** path. Solid Edge conversion replays **visual EMF primitives only** — no native layers, dimensions, CAM, or parametrics. See [docs/limitations.md](docs/limitations.md).

## Features

- **cncKad:** sections `[300]`/`[310]` geometry (native arcs/circles), `[100]`/`[200]`/`[210]`/`[500–503]` metadata, `[1100]`/`[1200]` CAM
- **Solid Edge:** CFB inspection, bounded EMF extraction, expanded record replay (lines, arcs, pens, transforms, text)
- **DXF:** native `ARC`/`CIRCLE`, layer table + ACI colors, `$INSUNITS` (mm), drawing extents, CAM layers (`PUNCH`/`CUT`/`TOOLS`)
- **SVG:** computed `viewBox` from bounds, per-layer groups, Y-flip for CAD coordinates
- **CLI:** single-file `convert`, batch `convert-all`, optional `--cam-json` sidecar, `--units` override

## What this project does **not** do

- Recover editable Solid Edge objects, constraints, or parametric model data
- Parse native PAR / PSM / ASM semantics
- Require Solid Edge, COM, Windows APIs, or commercial CAD runtimes
- Provide quotation, manufacturing, AI extraction, or business automation

## Output fidelity

| Source | Geometry | Layers / colors | Text | CAM | Material / thickness |
| --- | --- | --- | --- | --- | --- |
| cncKad | Native arcs / circles | ACI + layer ids | Planned | Yes (JSON + DXF layers) | Yes (metadata) |
| Solid Edge EMF | Lines, arcs, beziers (partial) | No | Basic | No | No |

Format details: [docs/cnckad-format.md](docs/cnckad-format.md) · Status matrix: [docs/IMPLEMENTATION-STATUS.md](docs/IMPLEMENTATION-STATUS.md)

## Supported platforms

Linux · Windows · macOS (pure Rust, no Solid Edge install required)

## Quick start

Requires Rust 1.88+ (see `rust-toolchain.toml`). On Windows, install
[Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
with the C++ workload so `link.exe` is available.

```bash
cargo build --release
```

### cncKad (typical workflow)

```bash
# Inspect part name, entity counts, sheet size
dft2dxf inspect part.DFT

# Convert to DXF + SVG preview + CAM/metadata JSON sidecar
dft2dxf convert part.DFT --output part.dxf --svg-preview ./preview --cam-json

# Validate all local fixtures (gitignored under tests/fixtures/valid/local/)
dft2dxf validate-fixtures --local

# Batch-convert every local .DFT → out/dxf/*.dxf and out/svg/<name>/sheet-1.svg
dft2dxf convert-all --local --dxf-dir ./out/dxf --svg-dir ./out/svg --cam-json
```

On Windows, use `.\target\release\dft2dxf.exe` if the binary is not on `PATH`.

### Solid Edge

```bash
# Inspect compound file structure and sheet metadata
dft2dxf inspect drawing.dft

# Extract embedded EMF for debugging
dft2dxf extract-emf drawing.dft --output-dir ./debug --sheet 1

# Convert sheet 1 to DXF (+ optional SVG)
dft2dxf convert drawing.dft --output output.dxf --sheet 1 --svg-preview ./preview
```

### CLI flags (global)

| Flag | Description |
| --- | --- |
| `--local` / `-l` | Use gitignored fixtures under `tests/fixtures/valid/local/` |
| `--cam-json` | Write `<output>.cam.json` with metadata + CAM program |
| `--units mm\|in\|unitless` | Override DXF drawing units |
| `--format json` | Machine-readable output for inspect / validate / convert-all |

## Library usage

### cncKad

```rust
use ckad_reader::read_to_drawing;

let drawing = read_to_drawing("part.DFT".as_ref(), ckad_reader::DEFAULT_MAX_FILE_SIZE)?;
println!("entities: {}", drawing.sheets[0].entities.len());
if let Some(cam) = &drawing.cam {
  println!("tools: {}, ops: {}", cam.tools.len(), cam.operations.len());
}
```

### Solid Edge

```rust
use dft_reader::{DftDocument, DftOpenOptions, Limits};

let options = DftOpenOptions::new().with_limits(Limits::strict());
let mut document = DftDocument::open_with_options("drawing.dft", options)?;
let sheets = document.sheets()?;
let emf = document.extract_emf(1)?;
emf.write_to("sheet-1.emf")?;
```

## Workspace layout

| Crate | Role |
| --- | --- |
| `ckad-reader` | cncKad text `.dft` → Drawing IR |
| `dft-reader` | Solid Edge CFB open, metadata, EMF extraction |
| `emf-reader` | EMF record replay → Drawing IR |
| `drawing-ir` | Shared geometry, metadata, CAM model |
| `drawing-dxf` / `drawing-svg` | DXF and SVG writers |
| `dft2dxf-cli` | Command-line interface |
| `dft2dxf-testkit` | Synthetic fixtures for tests |

## Contributing fixtures

Do not submit proprietary customer drawings without explicit written permission.

- **cncKad:** place private files in `tests/fixtures/valid/local/` (gitignored)
- **Solid Edge:** see [tests/fixtures/valid/INTAKE.md](tests/fixtures/valid/INTAKE.md) for redistributable fixture requirements

Also see [docs/test-fixtures.md](docs/test-fixtures.md) and [docs/ROADMAP.md](docs/ROADMAP.md).

## Test coverage

```bash
cargo test --workspace
bash scripts/coverage.sh   # Ubuntu / CI — enforces >= 80% line coverage
```

CI runs `cargo llvm-cov` on Ubuntu with `--fail-under-lines 80` (excludes `dft2dxf-testkit`).

## Security

DFT files are treated as untrusted input. Parsers apply limits to file sizes, stream sizes,
decompressed output, EMF record counts, and CFB traversal depth. Report security issues
privately through [SECURITY.md](SECURITY.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
