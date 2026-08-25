# [![eliware.org](https://eliware.org/logos/brand.png)](https://discord.gg/M6aTR9eTwN)

## galera-check [![license](https://img.shields.io/github/license/eliware/galera-check.svg)](LICENSE) [![build status](https://github.com/eliware/galera-check/actions/workflows/ci.yml/badge.svg)](https://github.com/eliware/galera-check/actions/workflows/ci.yml) [![Rust](https://img.shields.io/badge/rust-1.82%2B-orange.svg)](https://www.rust-lang.org/)

Small, dependency-free-at-runtime Rust health checker for MariaDB Galera
nodes. It is suitable for HAProxy external checks and can also be used from
cron, systemd, or Kubernetes operational tooling.

## Features

- Successful no-op when invoked without arguments.
- `--check` verifies `wsrep_local_state_comment=Synced`, `wsrep_ready=ON`, and
  `wsrep_cluster_status=Primary`.
- `--agent` serves standard HAProxy agent checks, returning `up` or `down` for
  the configured Galera node.
- `--agent --performance` serves weighted HAProxy agent checks using database
  and Galera performance metrics.
- Runtime URL or individually encoded connection variables; secrets are never compiled into the binary.
- Optional Rustls TLS support through URL options; production TLS policy is outside this package.
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

Display the supported commands or installed version:

```sh
./galera-check --help
./galera-check --version
```

The checker intentionally does nothing without arguments:

```sh
./galera-check
```

Run a Galera readiness check by supplying the connection URL at runtime:

```sh
GALERA_URL='mysql://user:password@10.0.0.81:3306' ./galera-check --check
```

Alternatively, provide individual connection variables:

```sh
GALERA_USER=user GALERA_PASSWORD=password GALERA_HOST=10.0.0.81 ./galera-check --check
```

Run an HAProxy agent listener with the node URL supplied at runtime:

```sh
GALERA_URL='mysql://user:password@10.0.0.81:3306' \
GALERA_AGENT_LISTEN='127.0.0.1:33060' ./galera-check --agent
```

The agent returns `up` only when the node is `Synced`, `wsrep_ready=ON`, and
`wsrep_cluster_status=Primary`, and
`down` for connection failures or unhealthy state. The listen address defaults
to `127.0.0.1:33060`.

Use performance mode only for a weighted read backend:

```sh
GALERA_URL='mysql://user:password@10.0.0.81:3306' \
GALERA_AGENT_LISTEN='127.0.0.1:33160' ./galera-check --agent --performance
```

Standard `--agent` mode remains health-only for ordered write/sequential-read
backends. Performance mode returns a dynamic weight percentage based on
Galera queues, flow control, and probe latency.

TLS is an optional library capability. It is not mandatory or certified for
the trusted-LAN production deployment; test any selected MySQL URL SSL mode
against the target MariaDB version before using it.

When both `GALERA_URL` and separate variables are present, `GALERA_URL` wins.
Separate variables are encoded safely, default the port to `3306`, and do not
provide URL-only TLS or timeout options.

There is currently no certified MariaDB/Galera compatibility matrix. The
intended future matrix is MariaDB 10.6 + Galera 4, MariaDB 10.11 + Galera 4,
MariaDB 11.4 + Galera 4, and the production MariaDB/Galera version.

Performance mode returns `down` when state, queue, flow-control, or latency
pressure produces an unsafe zero weight; it never advertises `up 0%`.

Set `GALERA_DIAGNOSTICS=1` for stderr reason categories (`config`, `connect`,
`auth`, `query`, `state`, or `performance`), or `GALERA_DIAGNOSTICS=json` for
the reason-code JSON form. Diagnostics never enter the HAProxy response and do
not include credentials. The opt-in integration harness is enabled with
`GALERA_CHECK_INTEGRATION=1` and operator-supplied `GALERA_INTEGRATION_*_URL`
variables; unset endpoints are skipped.

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

For a reproducible Debian 12 build with the required Rust and native build
dependencies:

```sh
docker build --tag galera-check-build:debian12 .
docker create --name galera-check-build galera-check-build:debian12
docker cp galera-check-build:/src/target/release/galera-check ./galera-check-linux-x86_64
docker rm galera-check-build
```

Before deployment, verify the checksum of the binary after copying it and
preserve executable mode (`0755`). Keep the previous binary available for
rollback, and do not deploy a file whose checksum does not match the release
artifact.

## Release and rollback

Releases are created from an explicitly authorized `v*` tag. The release
workflow builds Linux and Windows artifacts from that exact tag, publishes
SHA-256 files beside them, and publishes only after both platform builds pass.
For a deployment, verify the downloaded checksum before copying the binary,
record the source tag and checksum, and retain the prior binary so the service
can be restored without rebuilding.

## Development

```sh
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo llvm-cov --all-targets --all-features \
  --ignore-filename-regex 'src[/\\\\](agent|main|mysql_adapter)\\.rs' \
  --summary-only
cargo install cargo-audit --locked  # once, if cargo-audit is not installed
cargo audit
```

GitHub Actions runs formatting, checking, tests, Clippy, and coverage on
pushes and pull requests.

The coverage gate excludes the process wrapper (`src/main.rs`), agent listener
(`src/agent.rs`), and live MySQL adapter (`src/mysql_adapter.rs`). The
remaining CLI and library logic is required to remain at 100% for regions,
functions, and lines (100% x 3). Run
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
