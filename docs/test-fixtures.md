# Test Fixtures

## Directory layout

```text
tests/fixtures/
  valid/
  malformed/
  edge-cases/
  multi-sheet/
  text/
  dimensions/
  hatches/
  raster-content/
  transforms/
tests/golden/
  svg/
  dxf/
```

## Provenance requirements

Every committed real-world fixture must include:

- source and permission status
- Solid Edge version (if known)
- expected sheet count
- notes about content type (text, hatches, raster, etc.)
- redistribution allowance

## Synthetic fixtures

The workspace generates synthetic `.dft` and EMF files via `dft2dxf-testkit` for
unit, integration, and CLI tests. These are preferred for CI because they are small,
deterministic, and unencumbered.

## Real and local `.dft` fixtures

| Mode | Directory | How to enable |
| --- | --- | --- |
| Default (CI / redistributable) | `tests/fixtures/valid/` | `cargo test -p dft-reader opens_and_extracts_emf_from_real_solid_edge_dft_fixture` |
| Local proprietary (gitignored) | `tests/fixtures/valid/local/` | `cargo test … -- --local` or `DFT2DXF_LOCAL=1` |
| CLI batch validate | same as above | `dft2dxf validate-fixtures` or `dft2dxf validate-fixtures --local` |

See [tests/fixtures/valid/INTAKE.md](../tests/fixtures/valid/INTAKE.md).

## Contributing samples

If you can share anonymized samples, open an issue describing:

- Solid Edge version
- what the drawing contains
- whether redistribution in the repository is permitted

For the first **real** `.dft` fixture, follow [tests/fixtures/valid/INTAKE.md](../tests/fixtures/valid/INTAKE.md).

Do not attach proprietary customer files to public issues.
