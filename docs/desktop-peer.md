# Desktop peer lifecycle (QUEUE #82, E4 `#95`)

When `network_profile=P1`, `aira desktop start` supervises:

1. `aira-node --http` (unchanged)
2. `aira peer listen --bind <peer_listen> --recv` (loopback only)

When `network_profile=P2`, step 2 adds `--dht --apply-book` (opt-in DHT→address book).

`PeerPidRecord` includes `network_profile` for attach; profile change forces a new peer process.

Runtime files: `runtime/aira-peer.pid.json`, `aira-peer.lock`. Logs: `logs/aira-peer.*.log`.

`AIRA_BIN` or sibling of `aira-node` resolves the CLI. Stop tears down peer then node.

Non-loopback `peer_listen` is fail-closed until peer CLI `--explicit` is wired (Out of `#82`).
