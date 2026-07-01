# Fixture provenance

## Synthetic fixtures

| Fixture | Source | License | Notes |
| --- | --- | --- | --- |
| `dft2dxf-testkit` builders | Generated in tests | MIT OR Apache-2.0 | Preferred CI fixtures; not real Solid Edge output |
| `malformed/too-short-metadata.bin` | Hand-crafted bytes | MIT OR Apache-2.0 | Negative metadata tests |
| `malformed/negative-sheet-count.bin` | Hand-crafted bytes (18 B) | MIT OR Apache-2.0 | Matches `negative_sheet_count_metadata()` |

## Real Solid Edge fixture validation

**Status: blocked — fixture not yet committed.**

| Field | Value |
| --- | --- |
| Fixture filename | `real-solid-edge.dft` (expected path; file absent) |
| Source | *Pending — see [INTAKE.md](INTAKE.md)* |
| Redistribution status | *Not committed — permission not established* |
| Known Solid Edge version | *Unknown* |
| SHA-256 | *N/A until file committed* |
| Expected sheet count | *TBD after inspect* |
| Expected extracted EMF sheet count | *TBD (minimum: 1)* |
| Extracted EMF validation method | Automated: `validate_emf_header` + `is_emf()` in `real_fixture.rs` smoke test |
| Known limitations | Extraction-only M1 scope; no visual fidelity claims |

### Independent EMF validation (manual, optional)

| Field | Value |
| --- | --- |
| Validation tool | *Not run — no extracted real fixture EMF available* |
| Tool version | *N/A* |
| Platform | Windows 10 (audit environment) |
| Validation result | *Blocked pending real fixture* |
| Rendering limitations | External viewers may not match Solid Edge sheet coordinates |

### Redacted structure summary

*No real fixture committed. After intake, record:*

- CFB top-level storage names (redacted if sensitive)
- `JDraftViewerInfo` / `JDraftDocumentInfo` presence
- Sheet count and per-sheet stream numeric names
- Declared vs decompressed EMF sizes (approximate)
- Deviations from synthetic `dft2dxf-testkit` layout

## Contributing real fixtures

See [INTAKE.md](INTAKE.md) and [docs/test-fixtures.md](../../docs/test-fixtures.md).

Do not submit proprietary customer drawings without explicit written permission.
