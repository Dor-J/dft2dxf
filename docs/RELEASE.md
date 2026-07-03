# Release checklist

## Pre-release

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `bash scripts/coverage.sh` (≥ 80% line coverage)
- [ ] `cargo deny check`
- [ ] Update `CHANGELOG.md` with version and date
- [ ] Tag release: `git tag v0.2.0`

## Artifacts

| Artifact | Build command |
| --- | --- |
| CLI | `cargo build --release -p dft2dxf-cli` |
| Sidecar | `cargo build --release -p dft2dxf-sidecar` |

## Docker

```bash
docker build -f deploy/Dockerfile -t dft2dxf-sidecar:latest .
docker compose -f deploy/docker-compose.yml up
```

## Post-release

- [ ] Publish crates to crates.io (when ready)
- [ ] Update docs.rs metadata if API changed
