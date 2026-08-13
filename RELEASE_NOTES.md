# Release notes

## 0.1.0

Initial release baseline for `galera-check`, a small Rust CLI and library for
checking MariaDB Galera readiness.

- Provides a successful no-op when invoked without arguments.
- Adds `--check` readiness validation for `wsrep_local_state_comment=Synced`
  and `wsrep_ready=ON`.
- Reads the database URL and credentials at runtime, without compiling secrets
  into the binary.
- Supports encrypted MySQL connections through the bundled Rustls backend.
- Preserves useful exit codes for healthy, unhealthy, and invalid states.
- Separates CLI behavior, URL/check orchestration, readiness parsing, and
  MySQL transport into focused library modules.
- Includes deterministic unit and CLI integration coverage without requiring a
  live database.
- Establishes formatting, compilation, tests, Clippy, and strict coverage
  validation in GitHub Actions.
