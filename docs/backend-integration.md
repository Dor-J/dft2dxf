# Backend Integration

How to embed `dft2dxf` in a **FastAPI** (or other Python) backend. The core converter is Rust;
there is no published Python package today. Two supported integration patterns are documented below.

| Pattern | Status | Best for |
| --- | --- | --- |
| **CLI subprocess** | **Ready now** | Single-server deployments, batch jobs, simplest ops |
| **HTTP sidecar (Axum)** | **Planned** (M9) | Many concurrent conversions, shared worker pool, multi-language clients |

Both patterns treat uploaded `.dft` files as **untrusted input** and rely on the same bounded
parsers documented in [security-model.md](security-model.md).

---

## Architecture overview

```text
┌─────────────────────────────────────────────────────────────────┐
│  FastAPI app (Python)                                           │
│  ┌──────────────┐    upload / job queue    ┌─────────────────┐  │
│  │ REST routes  │ ───────────────────────► │ temp storage    │  │
│  └──────┬───────┘                          └────────┬────────┘  │
│         │                                           │           │
│         │  Option A: subprocess                     │           │
│         ▼                                           ▼           │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  dft2dxf convert …  (release binary on PATH)             │   │
│  └──────────────────────────────────────────────────────────┘   │
│         │                                                       │
│         │  Option B: HTTP (planned)                             │
│         ▼                                                       │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  dft2dxf-sidecar (Axum) — conversion worker pool         │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
         │
         ▼
   DXF / SVG / CAM JSON  →  object storage or direct HTTP response
```

**Data flow inside the converter** (unchanged for both options):

```text
.dft  →  ckad-reader | dft-reader  →  drawing-ir  →  drawing-dxf / drawing-svg
```

See [architecture.md](architecture.md) for crate responsibilities.

---

## Option A — CLI subprocess (recommended today)

Call the `dft2dxf` release binary from FastAPI. Each request (or background job) writes the upload
to a temp directory, invokes `convert`, and reads the output files.

### Prerequisites

1. Build the binary once per deployment target:

   ```bash
   cargo build --release
   ```

2. Install `target/release/dft2dxf` (or `dft2dxf.exe` on Windows) on `PATH`, or set an explicit
   path via configuration.

3. Run conversion in a **background worker** (Celery, RQ, `asyncio` task pool, or similar).
   Parsing is CPU-bound; avoid blocking the event loop for large files.

### Example service (FastAPI + asyncio)

```python
import asyncio
import json
import shutil
import tempfile
from dataclasses import dataclass
from pathlib import Path

DFT2DXF_BIN = Path("/usr/local/bin/dft2dxf")  # or from settings


@dataclass(frozen=True)
class ConvertResult:
    dxf: bytes
    cam_json: dict | None
    svg: bytes | None


async def convert_dft(
    dft_bytes: bytes,
    *,
    sheet: int | None = None,
    include_svg: bool = False,
    include_cam_json: bool = True,
) -> ConvertResult:
    """Convert an uploaded .dft to DXF (and optional SVG / CAM JSON)."""
    with tempfile.TemporaryDirectory(prefix="dft2dxf-") as tmp:
        root = Path(tmp)
        dft_path = root / "input.dft"
        dxf_path = root / "output.dxf"
        svg_dir = root / "svg"

        dft_path.write_bytes(dft_bytes)

        cmd: list[str] = [
            str(DFT2DXF_BIN),
            "convert",
            str(dft_path),
            "--output",
            str(dxf_path),
        ]
        if sheet is not None:
            cmd.extend(["--sheet", str(sheet)])
        if include_svg:
            cmd.extend(["--svg-preview", str(svg_dir)])
        if include_cam_json:
            cmd.append("--cam-json")

        proc = await asyncio.create_subprocess_exec(
            *cmd,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        stdout, stderr = await proc.communicate()

        if proc.returncode != 0:
            detail = stderr.decode("utf-8", errors="replace") or stdout.decode(
                "utf-8", errors="replace"
            )
            raise RuntimeError(f"dft2dxf failed (exit {proc.returncode}): {detail}")

        cam: dict | None = None
        cam_path = dxf_path.with_suffix(".cam.json")
        if include_cam_json and cam_path.exists():
            cam = json.loads(cam_path.read_text(encoding="utf-8"))

        svg_bytes: bytes | None = None
        if include_svg and svg_dir.exists():
            # cncKad → sheet-1.svg; Solid Edge → sheet-{n}.svg
            svg_files = sorted(svg_dir.glob("*.svg"))
            if svg_files:
                svg_bytes = svg_files[0].read_bytes()

        return ConvertResult(
            dxf=dxf_path.read_bytes(),
            cam_json=cam,
            svg=svg_bytes,
        )
```

### Example route

```python
from fastapi import FastAPI, File, HTTPException, UploadFile
from fastapi.responses import Response

app = FastAPI()


@app.post("/convert/dxf")
async def convert_to_dxf(file: UploadFile = File(...)) -> Response:
    if not file.filename or not file.filename.lower().endswith(".dft"):
        raise HTTPException(status_code=400, detail="Expected a .dft file")

    try:
        result = await convert_dft(await file.read(), include_cam_json=True)
    except FileNotFoundError:
        raise HTTPException(status_code=503, detail="Converter binary not installed")
    except RuntimeError as exc:
        raise HTTPException(status_code=422, detail=str(exc))

    return Response(
        content=result.dxf,
        media_type="application/dxf",
        headers={"Content-Disposition": f'attachment; filename="{Path(file.filename).stem}.dxf"'},
    )
```

### Operational notes

| Topic | Guidance |
| --- | --- |
| **Binary location** | Pin path in env (`DFT2DXF_BIN`); verify at startup with `dft2dxf --version`. |
| **Concurrency** | Limit parallel subprocesses (semaphore) to avoid CPU/memory spikes. |
| **Timeouts** | Wrap `communicate()` with `asyncio.wait_for(..., timeout=120)`. |
| **Inspect / validate** | Use `dft2dxf inspect --format json` or `validate --format json` before convert. |
| **Batch** | Use `convert-all --format json` for directory jobs; parse JSON summary from stdout. |
| **Limits** | Pass `--max-file-size`, `--max-decompressed-size` for stricter caps on public uploads. |
| **Docker** | Multi-stage build: Rust builder stage → copy `dft2dxf` into Python runtime image. |

### Format support reminder

| Source | Production readiness |
| --- | --- |
| **cncKad** | Primary path — geometry, layers, metadata, CAM JSON |
| **Solid Edge** | Visual EMF replay only — preview/SVG OK; do not expect full CAD fidelity |

See [limitations.md](limitations.md).

---

## Option B — HTTP sidecar (Axum, implemented)

For backends that need **many concurrent conversions** without spawning a subprocess per request,
run the **`dft2dxf-sidecar`** service. It calls `dft2dxf-core` in-process with a bounded worker pool.

### Why a sidecar

| Benefit | Detail |
| --- | --- |
| **Shared worker pool** | Tokio + bounded queue; reuse parsers and allocators across requests |
| **No subprocess overhead** | In-process `ckad-reader` / `dft-reader` → `drawing-ir` → writers |
| **Language-agnostic** | Any client (Python, Node, Go) can POST multipart uploads |
| **Independent scaling** | Scale converter replicas separately from the API tier |

### Planned crate layout

```text
crates/dft2dxf-sidecar/
  src/main.rs       # Axum server, routes, graceful shutdown
  src/convert.rs    # Shared logic from dft2dxf-cli (format detect → IR → writers)
  src/pool.rs       # Semaphore-limited conversion pool
  src/error.rs      # HTTP error mapping
```

The sidecar will **not** reimplement parsing; it will depend on `dft2dxf-cli` library functions
(or a extracted `dft2dxf-core` module) plus `drawing-dxf`, `drawing-svg`, and `drawing-ir`.

### Planned API (draft)

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/health` | Liveness + version |
| `GET` | `/ready` | Warmup complete, pool accepting jobs |
| `POST` | `/v1/convert` | Multipart: `file` (.dft), optional `sheet`, `units`, flags |
| `POST` | `/v1/inspect` | Multipart: `file` → JSON metadata summary |
| `POST` | `/v1/validate` | Multipart: `file` → validation result |

**HTTP status codes:** `400` invalid upload; `422` conversion/validation failure; `503` worker pool at capacity (`WORKER_CONCURRENCY` exhausted) or shutting down.

**`POST /v1/convert` response (multipart or JSON envelope):**

```json
{
  "format": "cnckad",
  "sheet": 1,
  "entities": 142,
  "diagnostics": [],
  "outputs": {
    "dxf_base64": "...",
    "svg_base64": "...",
    "cam_json": { "metadata": {}, "cam": {} }
  }
}
```

Large responses may use **streaming** or presigned object-storage URLs instead of inline base64.

### FastAPI client (sidecar, planned)

```python
import httpx

SIDECAR_URL = "http://127.0.0.1:8080"


async def convert_via_sidecar(dft_bytes: bytes, filename: str = "input.dft") -> dict:
    async with httpx.AsyncClient(timeout=120.0) as client:
        response = await client.post(
            f"{SIDECAR_URL}/v1/convert",
            files={"file": (filename, dft_bytes, "application/octet-stream")},
            data={"include_svg": "true", "include_cam_json": "true"},
        )
        response.raise_for_status()
        return response.json()
```

### Deployment sketch

```text
┌─────────────┐     HTTP      ┌──────────────────┐
│  FastAPI    │ ────────────► │  dft2dxf-sidecar │  × N replicas
│  (Python)   │   private net │  (Axum / Rust)   │
└─────────────┘               └──────────────────┘
```

- Run sidecar on `127.0.0.1` in the same pod (sidecar container) or as a separate service.
- Configure `WORKER_CONCURRENCY` to match CPU cores.
- Use the same `Limits` defaults as the CLI (`Limits::strict()`).

### M9 acceptance criteria

- [x] `dft2dxf-sidecar` crate with `/health`, `/v1/convert`, `/v1/inspect`, `/v1/validate`
- [x] Bounded conversion pool (`WORKER_CONCURRENCY` env var)
- [x] Integration test: HTTP POST → valid DXF base64
- [x] Docker Compose example (`deploy/docker-compose.yml`)
- [x] Document env vars and resource limits

Build and run:

```bash
cargo build --release -p dft2dxf-sidecar
WORKER_CONCURRENCY=4 ./target/release/dft2dxf-sidecar
# or: docker compose -f deploy/docker-compose.yml up
```

---

## Choosing a pattern

| Criterion | CLI subprocess | HTTP sidecar |
| --- | --- | --- |
| Time to integrate | **Hours** | Days (after M9 ships) |
| Peak concurrency | Limited by process spawn cost | **Better** — shared pool |
| Operational complexity | **Low** | Medium (second service) |
| Memory profile | One process per job | **Better** — amortized |
| cncKad production use | **Yes** | Yes (when built) |
| Solid Edge preview | Yes | Yes (when built) |

**Recommendation:** Start with **CLI subprocess** for cncKad production workflows. Adopt the
**Axum sidecar** when subprocess overhead or concurrent load becomes a bottleneck.

---

## Related documentation

- [IMPLEMENTATION-STATUS.md](IMPLEMENTATION-STATUS.md) — capability matrix and milestone progress
- [limitations.md](limitations.md) — fidelity ceilings per input format
- [security-model.md](security-model.md) — untrusted input limits
- [ROADMAP.md](ROADMAP.md) — M9 backend integration milestone
