# Changelog

## Unreleased

- Refactored CLI behavior into testable library logic with injectable check
  seams, covering healthy and failure paths without requiring a live database.
- Restored the release coverage gate to 100% for regions, functions, and lines,
  with exclusions limited to the process-exit wrapper and MySQL transport.
- Refined the project layout into a reusable library plus CLI binary.
- Added metadata, licensing, unit coverage, and CI validation.
- Reject extra command-line arguments with exit code `2`.
- Added CLI integration tests and deterministic connection-failure coverage.
- Updated GitHub Actions checkout and cache actions to their Node.js 24
  versions.
