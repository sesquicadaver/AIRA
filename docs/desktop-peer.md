# Desktop peer lifecycle (QUEUE #82, E4 `#95`, `#98`)

**Phase G:** CI stabilization for P3/P4 integration tests (`#131`–`#132`); see [`phase-g-plan.md`](phase-g-plan.md).

When `network_profile=P1`, `aira desktop start` supervises:

1. `aira-node --http` (unchanged)
2. `aira peer listen --bind <peer_listen> --recv` (loopback only)

When `network_profile=P2`, step 2 adds `--dht --apply-book` (opt-in DHT→address book).

When `network_profile=P3`, step 2 is `peer listen --bind <peer_listen> --relay --relay-ttl-days N` (no `--recv`; relay hub mode).

When `network_profile=P4`, step 2 is `peer listen --bind <peer_listen> --recv --dht --apply-book --apply-trust --gossip` (no `--relay`).

`PeerPidRecord` includes `network_profile` and P3 `relay_ttl_days` for attach; profile/TTL change forces a new peer process.

Runtime files: `runtime/aira-peer.pid.json`, `aira-peer.lock`. Relay registry: `peers/relay_hub.json`. Logs: `logs/aira-peer.*.log`.

`AIRA_BIN` or sibling of `aira-node` resolves the CLI. Stop tears down peer then node.

Non-loopback `peer_listen` is fail-closed until peer CLI `--explicit` is wired (Out of `#82`).

## Opt-in dial / session evidence (Phase S `#300`)

Desktop **Technical details → Peer dial** calls `run_opt_in_peer_dial` (trusted peer + **explicit** `dial_addr`):

1. Upsert AddressBook with the operator-supplied address
2. `aira_peer::dial` (hello + Noise XX)
3. Persist `peers/dial_session_evidence.json` (handshake hash + endpoint + timestamp)
4. Mesh projection may set `live_session_count = Some(1)` while evidence is fresh — **without** applying reachability DIRECT

Setup / invite / Stop→Start alone still leave live sessions unobserved (`None`). Public bind is not the Desktop default.

Profile matrix and RFC index: [`desktop-network-profiles.md`](desktop-network-profiles.md).
RFC: [`AIRA-RFC-0187`](../specs/rfc/AIRA-RFC-0187-opt-in-peer-dial-session-evidence.md).
