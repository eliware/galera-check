# galera-check

Small, dependency-free-at-runtime Rust health checker for MariaDB Galera nodes.

It is suitable for HAProxy external checks and can also be used from cron,
systemd, or Kubernetes operational tooling.

It is intentionally a no-op without arguments. Pass `--check` to connect using
the `GALERA_URL` environment variable and require:

- `wsrep_local_state_comment=Synced`
- `wsrep_ready=ON`

The URL is supplied at runtime so credentials are not compiled into the
binary:

```sh
GALERA_URL='mysql://user:password@10.0.0.81:3306' ./galera-check --check
```

The URL may include standard MySQL connection options supported by the Rust
MySQL driver. Keep credentials in the service environment or a protected
secret file; do not commit them to a repository.

## Exit behavior

- No arguments: exit successfully without connecting. This lets the same file
  be installed as a deployment hook and an external checker.
- `--check`: connect and require `wsrep_local_state_comment=Synced` and
  `wsrep_ready=ON`.
- Invalid arguments or missing configuration: exit `2`.
- Connection failures or unhealthy Galera state: exit `1`.

## HAProxy

HAProxy external checks provide `HAPROXY_SERVER_ADDR` and
`HAPROXY_SERVER_PORT`, but this binary intentionally accepts a complete URL so
it can be tested independently. A wrapper can preserve the configured
credentials while substituting the backend address for each check.

## Development

Build a release binary with:

```sh
cargo build --release
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

The GitHub Actions workflow runs formatting, checking, tests, and Clippy on
pushes and pull requests.
