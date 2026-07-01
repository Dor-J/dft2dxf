# cncKad text `.dft` format (decoded subset)

Metalix cncKad saves sheet-metal parts as a **line-oriented text file** (ASCII or UTF-16 LE
with BOM). Files begin with `gKad` or `CKad`. This document summarizes sections parsed by
`ckad-reader` as of the professional conversion milestone.

## Header

```text
gKad 9.80
cncKad Version 95276
None
```

## Section index

| Section | Purpose |
| --- | --- |
| `[100]` | Part name, customer, bar code fields |
| `[200]` | Sheet setup: `/E` extents, `/P` scale, `/M` material code |
| `[210]` | `KFactor` bend factor |
| `[300]` | Primary geometry (lines, circles, arcs) |
| `[310]` | Secondary geometry block (same grammar as `[300]`) |
| `[500]`–`[503]` | Thickness and sheet parameters (`[503]` often holds thickness mm) |
| `[1100]` | Tool table (`R`, `C`, … + `TOOLCM` comments) |
| `[1200]` | CAM operations (`ONLINE`, `SINGLE`, `ONARC`) |

## Geometry grammar

Each geometry block contains sub-blocks:

### `LINES`

```text
LINES
<count>
<x1> <y1> <x2> <y2> ...
<metadata line>
```

Metadata line (example `1 0 269 0 15 0`):

- Field 3 (0-based index 2): layer id → exported as `L{id}`
- Field 5 (index 4): AutoCAD Color Index (ACI) for stroke color

### `CIRCLES`

Single-line records:

```text
<cx> <cy> <radius> <aci> ...
```

### `ARCS`

Three lines per arc:

```text
<cx> <cy> <radius> <aci> ...
<startX> <startY> <endX> <endY>
<startDeg> <endDeg>
```

Arcs are emitted as native `Arc` IR entities (not polylines).

### `OLE4DM`

Extension lines beginning with `OLE4DM` are skipped during geometry parsing.

## CAM

### Tool table `[1100]`

```text
R 50 5
...
TOOLCM "(M)"
C 10
...
```

Parsed into `CamProgram.tools` with kind, size, and comment.

### Operations `[1200]`

| Keyword | Meaning |
| --- | --- |
| `ONLINE` | Linear cut path |
| `SINGLE` | Punch hit (position parsed from coordinate line when possible) |
| `ONARC` | Arc cut |

Operations are stored losslessly in `CamProgram.operations` with raw lines. DXF export
adds `CUT`/`PUNCH`/`TOOLS` layers; `--cam-json` writes a sidecar file.

## Units

Coordinates are **millimeters**. DXF export sets `$INSUNITS=4` (mm) and `$MEASUREMENT=1`
(metric).

## Encoding

| Signal | Decoder |
| --- | --- |
| BOM `FF FE` | UTF-16 LE |
| Otherwise | Latin-1 (byte → char) |

## Example workflow

```bash
dft2dxf inspect tests/fixtures/valid/local/102-00.DFT
dft2dxf convert part.DFT --output part.dxf --svg-preview out/svg --cam-json
dft2dxf convert-all --local --dxf-dir out/dxf --svg-dir out/svg --cam-json
```
