# VyOS deployment notes

Configure `GALERA_URL` (preferred) or the separately encoded
`GALERA_USER`, `GALERA_PASSWORD`, `GALERA_HOST`, and optional `GALERA_PORT`
variables. Keep the agent bound to loopback unless an explicit firewall and
HAProxy design requires another address.

The health response is exactly `up\n` or `down\n`; performance mode is
`up N%\n` and returns `down\n` for an unsafe zero weight. `Synced`, `ON`, and
`Primary` are all required.

Verify an installation with `galera-check --version`, then query the listener
with `printf '\n' | nc 127.0.0.1 33060`. Verify the binary checksum against the
release checksum before installation.

For rollback, stop the service, restore the retained previous binary with mode
0755, verify its checksum, and restart the service. Do not overwrite the last
known-good copy until the new artifact has passed local and router-side checks.

No MariaDB/Galera compatibility matrix is currently certified. Treat changes
in MariaDB, Galera, URL SSL options, and performance thresholds as deployment
risks requiring staging validation.
