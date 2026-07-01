# DFT Format Research

## Known viewer storage layout

Forensic evidence from archived `SolidEdge.Community.Reader` suggests:

- Storage: `JDraftViewerInfo`
- Metadata stream: `JDraftDocumentInfo`
- Per-sheet streams: `1`, `2`, `3`, ... (one-based numeric names)
- Sheet payload: zlib-compressed EMF bytes

## Metadata structures under investigation

`DRAFTDOCUMENTINFO` (observed fields):

- `viewer_info_version: u32`
- `number_of_sheets: i32`
- `active_sheet_index: i32`
- `geometric_version: u32`
- `units: u16`

Per sheet:

- UTF-16LE name length (code units) and name bytes
- `width: f64`
- `height: f64`
- `emf_size: u32`
- `emf_compressed_size: u32`

## Open questions requiring real fixtures

- Exact struct padding for all Solid Edge versions
- Whether all DFT files contain `JDraftViewerInfo`
- Whether compression is always zlib-wrapped DEFLATE
- How background sheets and title blocks are represented

## Validation approach

1. Run `dft2dxf inspect` on permitted fixtures
2. Extract EMF with `dft2dxf extract-emf`
3. Validate EMF independently with external viewers/renderers
4. Record findings in fixture provenance notes
