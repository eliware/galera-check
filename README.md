# [![eliware.org](https://eliware.org/logos/brand.png)](https://discord.gg/M6aTR9eTwN)

## galera-check [![license](https://img.shields.io/github/license/eliware/galera-check.svg)](LICENSE) [![build status](https://github.com/eliware/galera-check/actions/workflows/ci.yml/badge.svg)](https://github.com/eliware/galera-check/actions/workflows/ci.yml) [![Rust](https://img.shields.io/badge/rust-1.82%2B-orange.svg)](https://www.rust-lang.org/)

Small, dependency-free-at-runtime Rust health checker for MariaDB Galera
nodes. It is suitable for HAProxy external checks and can also be used from
cron, systemd, or Kubernetes operational tooling.

## Features

- Successful no-op when invoked without arguments.
- `--check` verifies `wsrep_local_state_comment=Synced` and
  `wsrep_ready=ON`.
- Runtime URL and credentials; secrets are never compiled into the binary.
- Rustls TLS support for encrypted database connections.
- Useful exit codes for HAProxy and automation.
- Small release binary with no runtime dependency on Rust.

## Project structure

The implementation is split into focused modules:

- `src/cli.rs` handles argument parsing and maps failures to exit codes.
- `src/checker.rs` parses the database URL and coordinates a check.
- `src/status.rs` parses Galera status rows and applies readiness rules.
- `src/mysql_adapter.rs` contains the MySQL connection and query code.
- `src/lib.rs` exposes the library API; `src/main.rs` handles the process
  boundary.

Status parsing and readiness validation are tested independently from the
database transport. Process-level behavior is covered in `tests/cli.rs`.

## Usage

The checker intentionally does nothing without arguments:

```sh
./galera-check
```

Run a Galera readiness check by supplying the connection URL at runtime:

```sh
GALERA_URL='mysql://user:password@10.0.0.81:3306' ./galera-check --check
```

TLS can be requested with MySQL URL options such as
`?ssl-mode=REQUIRED`; the bundled Rustls backend validates the server
certificate using the platform root store.

Keep credentials in the service environment or a protected secret file. Never
commit real credentials to the repository.

Exit behavior:

- No arguments: `0`, without connecting.
- Healthy `--check`: `0`.
- Connection failure or unhealthy Galera state: `1`.
- Invalid arguments or configuration, including a missing or malformed
  `GALERA_URL`: `2`.

## HAProxy

HAProxy external checks provide `HAPROXY_SERVER_ADDR` and
`HAPROXY_SERVER_PORT`, but this binary accepts a complete URL so it can be
tested independently. A wrapper can preserve configured credentials while
substituting the backend address for each check.

## Installation

Build from source:

```sh
git clone https://github.com/eliware/galera-check.git
cd galera-check
cargo build --release
install -m 0755 target/release/galera-check /usr/local/bin/galera-check
```

## Development

```sh
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo llvm-cov --all-targets --all-features \
  --ignore-filename-regex 'src/(main|mysql_adapter).rs' \
  --summary-only
```

GitHub Actions runs formatting, checking, tests, Clippy, and coverage on
pushes and pull requests.

The coverage gate excludes only the process wrapper (`src/main.rs`) and live
MySQL adapter (`src/mysql_adapter.rs`). The remaining CLI and library logic is
required to remain at 100% for regions, functions, and lines (100% x 3). Run
the command above to reproduce the CI coverage gate. Transport-independent
behavior is tested with local fakes. The live-check integration test is opt-in:
set `GALERA_CHECK_LIVE=1` and provide an explicit `GALERA_URL`. Never commit or
log that URL.

## Support

For help or community discussion, visit eliware.org on Discord.

[![Discord](https://eliware.org/logos/discord_96.png)](https://discord.gg/M6aTR9eTwN)
**[eliware.org on Discord](https://discord.gg/M6aTR9eTwN)**

## License

[MIT © 2026 Eli Sterling, eliware.org](LICENSE)

## Links

- [Project Home](https://eliware.org)
- [GitHub Repo](https://github.com/eliware/galera-check)
- [GitHub Org](https://github.com/eliware)
- [GitHub Personal](https://github.com/eli-sterling)
- [Discord](https://discord.gg/M6aTR9eTwN)
