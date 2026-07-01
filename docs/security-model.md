# Security Model

## Defaults

Configured in `dft_reader::Limits::strict()`:

- `max_file_size`: 256 MiB
- `max_stream_size`: 64 MiB
- `max_decompressed_size`: 256 MiB
- `max_sheet_count`: 1024
- `max_storage_depth`: 32
- `max_entry_count`: 100000

EMF parsing defaults:

- `max_record_count`: 1,000,000
- `max_record_size`: 16 MiB

## Rules

- All offsets and lengths are validated before slicing
- Decompression is bounded and reports limit errors deterministically
- Malformed files must not panic parsing code
- No external converter subprocesses are invoked by default

## Dependency policy

Core path dependencies must be permissively licensed (MIT/Apache-2.0/BSD-compatible).
`cargo-deny` enforces license and advisory checks in CI.
