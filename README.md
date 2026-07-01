# dft2dxf

Convert Solid Edge `.dft` draft files into portable vector outputs by extracting the
embedded viewer EMF representation and replaying it into a canonical drawing IR.

## What this project does

- Opens `.dft` files as cross-platform Compound File Binary containers
- Inspects storage structure and draft viewer metadata
- Safely extracts embedded per-sheet EMF streams
- Replays a growing subset of EMF graphics records into Drawing IR
- Writes SVG previews and experimental DXF output

## What this project does **not** do

- Recover editable Solid Edge objects, constraints, or parametric model data
- Parse native PAR / PSM / ASM semantics
- Require Solid Edge, COM, Windows APIs, or commercial CAD runtimes
- Provide quotation, manufacturing, AI extraction, or business automation

## Supported platforms

- Linux
- Windows
- macOS

## Maturity

Early development. The first implementation milestone is safe extraction of embedded per-sheet EMF streams. DXF conversion is
experimental and fidelity varies by drawing content.

## Compatibility

Support is based on extracting embedded Solid Edge viewer EMF data. Files that do
not contain compatible viewer streams, use unsupported compression/layout variants,
or rely on non-embedded background-sheet content may not convert successfully.

The project reports unsupported structures explicitly rather than silently producing
incomplete output.

## Output fidelity

SVG is the primary validation output. DXF conversion maps visible EMF graphics into
portable DXF entities where possible.

Text, fills, clipping, embedded rasters, custom line styles, complex transforms,
and unsupported EMF records may have reduced fidelity or be emitted as diagnostics.

## Security

DFT files are treated as untrusted input. The parser applies limits to file sizes,
streams, decompression output, EMF record counts, and allocation sizes. Please report
security issues privately through [SECURITY.md](SECURITY.md).

## Quick start

Requires Rust 1.88+ (see `rust-toolchain.toml`). On Windows, install
[Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
with the C++ workload so `link.exe` is available.

```bash
cargo build --release

# Inspect compound file structure and sheet metadata
dft2dxf inspect drawing.dft

# Extract embedded EMF for validation
dft2dxf extract-emf drawing.dft --output-dir ./debug --sheet 1

# Experimental conversion to DXF
dft2dxf convert drawing.dft --output output.dxf --sheet 1 --svg-preview ./preview
```

## Library usage

```rust
use dft_reader::{DftDocument, DftOpenOptions, Limits};

let options = DftOpenOptions::new().with_limits(Limits::strict());
let mut document = DftDocument::open_with_options("drawing.dft", options)?;
let sheets = document.sheets()?;
let emf = document.extract_emf(1)?;
emf.write_to("sheet-1.emf")?;
```

## Contributing fixtures

Do not submit proprietary customer drawings without explicit written permission.
Prefer synthetic or anonymized samples. See [docs/test-fixtures.md](docs/test-fixtures.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
