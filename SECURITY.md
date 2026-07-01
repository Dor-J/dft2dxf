# Security Policy

## Supported versions

Security fixes are provided for the latest `main` branch.

## Reporting a vulnerability

Please report security issues privately through your organization's preferred
private disclosure channel. Do not open public issues for exploit details.

## Threat model

All `.dft` uploads are treated as hostile. The project enforces:

- file-size limits
- stream-size limits
- decompression output limits
- sheet and record count limits
- checked offset/length validation

See [docs/security-model.md](docs/security-model.md) for details.
