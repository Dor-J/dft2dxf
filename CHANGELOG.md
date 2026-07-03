# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-07-03

### Added

- Structured `EmfHeader` parsing with `nBytes` and bounds validation (M2).
- `Deserialize` on all Drawing IR public types; `docs/ir-schema.md` (M3).
- Golden SVG pipeline tests for rectangle, polyline, polygon, moveto/lineto, text (M4).
- EMF pen/select stroke validation tests and corrected `EMR_CREATEPEN` layout (M5).
- DXF golden tests, `write_drawing_to_bytes`, updated `docs/dxf-mapping.md` (M6).
- Proper `EMRTEXT` parsing; diagnostics for clipping, fills, raster EMF records (M7).
- CI fuzz-smoke job; `docs/RELEASE.md` (M8).
- `dft2dxf-core` in-memory conversion library (M9).
- `dft2dxf-sidecar` Axum HTTP service with `/health`, `/ready`, `/v1/convert`, `/v1/inspect`, `/v1/validate` (M9).
- `deploy/Dockerfile` and `deploy/docker-compose.yml`.

### Changed

- `EmfDocument` requires a valid `EMR_HEADER` as the first record.
- Testkit EMF moveto/lineto and text record layouts aligned with Win32 EMF structures.

[Unreleased]: https://github.com/dft2dxf/dft2dxf/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/dft2dxf/dft2dxf/compare/v0.1.0...v0.2.0

### Added

- cncKad text `.dft` parser with geometry, metadata, and CAM.
- Solid Edge CFB reader with bounded EMF extraction.
- EMF record replay into Drawing IR.
- DXF and SVG writers.
- CLI: `convert`, `convert-all`, `inspect`, `validate`, `--cam-json`, `--format json`.

[Unreleased]: https://github.com/dft2dxf/dft2dxf/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/dft2dxf/dft2dxf/releases/tag/v0.1.0
