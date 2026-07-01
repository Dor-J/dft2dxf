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

## Contributing samples

If you can share anonymized or synthetic DFT samples, open an issue describing:

- Solid Edge version
- what the drawing contains
- whether redistribution in the repository is permitted

Do not attach proprietary customer files to public issues.
