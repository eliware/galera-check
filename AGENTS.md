# AGENTS.md

## Project

`galera-check` is a small Rust CLI and library for checking MariaDB Galera
readiness. The binary is designed to run as an HAProxy external check or as a
standalone operational diagnostic.

## Layout

- `src/lib.rs` — small public library API and module wiring
- `src/main.rs` — process environment, output, and exit handling
- `src/cli.rs` — argument parsing and CLI result/exit-code mapping
- `src/checker.rs` — URL parsing and checker orchestration
- `src/status.rs` — Galera status extraction and readiness validation
- `src/mysql_adapter.rs` — MySQL connection and status query implementation
- `tests/cli.rs` — process-level CLI behavior tests
- `Cargo.toml` / `Cargo.lock` — dependency and package metadata
- `.github/workflows/ci.yml` — formatting, checking, tests, Clippy, and coverage

## Behavior

- No arguments must remain a successful no-op.
- `--check` reads `GALERA_URL` and requires `wsrep_local_state_comment=Synced`
  and `wsrep_ready=ON`.
- Do not compile credentials into the binary or commit credentials to Git.
- Preserve useful exit codes: `0` healthy/no-op, `1` health or connection
  failure, `2` invalid invocation or configuration.

## Validation

Run the following before committing:

```sh
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo llvm-cov --all-targets --all-features --summary-only
```

Live checks require an explicitly supplied `GALERA_URL`; never put that URL in
source, logs, fixtures, or documentation with real credentials.

## Changes and releases

- Keep the checker small and dependency changes justified.
- Keep database-specific behavior in `mysql_adapter.rs`; keep readiness rules
  and status parsing independent of the MySQL transport so they remain easy to
  test.
- Update tests and README behavior documentation when CLI behavior changes.
- Do not deploy binaries to routers or change HAProxy/VyOS configuration from
  this repository unless the user explicitly requests it.
- Do not commit build output under `target/` or local environment files.
