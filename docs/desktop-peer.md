# Desktop peer lifecycle (QUEUE #82, E4 `#95`, `#98`)

When `network_profile=P1`, `aira desktop start` supervises:

1. `aira-node --http` (unchanged)
2. `aira peer listen --bind <peer_listen> --recv` (loopback only)

When `network_profile=P2`, step 2 adds `--dht --apply-book` (opt-in DHT→address book).

When `network_profile=P3`, step 2 is `peer listen --bind <peer_listen> --relay --relay-ttl-days N` (no `--recv`; relay hub mode).

`PeerPidRecord` includes `network_profile` and P3 `relay_ttl_days` for attach; profile/TTL change forces a new peer process.

Runtime files: `runtime/aira-peer.pid.json`, `aira-peer.lock`. Relay registry: `peers/relay_hub.json`. Logs: `logs/aira-peer.*.log`.

`AIRA_BIN` or sibling of `aira-node` resolves the CLI. Stop tears down peer then node.

Non-loopback `peer_listen` is fail-closed until peer CLI `--explicit` is wired (Out of `#82`).
