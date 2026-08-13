# galera-check

Small Rust health checker for MariaDB Galera nodes.

It is intentionally a no-op without arguments. Pass `--check` to connect using
the `GALERA_URL` environment variable and require:

- `wsrep_local_state_comment=Synced`
- `wsrep_ready=ON`

Example:

```sh
GALERA_URL='mysql://user:password@10.0.0.81:3306' ./galera-check --check
```

Build a release binary with:

```sh
cargo build --release
```
