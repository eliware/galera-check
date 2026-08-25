# Release notes

## Unreleased

## 0.2.5 release candidate

- Source version: `0.2.5` (tag publication intentionally not performed here).
- Target artifact: Debian 12, Linux x86_64, built from the pinned Debian 12
  Docker base and Rust 1.98.0 toolchain.
- Artifact: `target/release/galera-check-debian12-x86_64`.
- SHA-256: `9C4ED75581665726E7095E022F13DA2B83E482AA52AB7B0388E620A344391EC6`.
- Verification: formatting, check, test, Clippy, and 100% in-scope coverage
  gates passed; opt-in integration tests are skipped unless enabled.
- Artifact signing remains a follow-up deployment improvement.

- Added stable `--help` and `--version` CLI output.
- Added a Debian 12 Docker build definition for Linux x86_64 artifacts.
- Added cross-platform CI, dependency auditing, checksums, and deployment
  rollback guidance.

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
## 0.2.0

- Add `--agent` mode for HAProxy agent checks.
- Support a loopback TCP listener returning `up` or `down` from Galera readiness.
