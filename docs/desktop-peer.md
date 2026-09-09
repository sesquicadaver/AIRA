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

## Opt-in dial / session evidence (Phase S `#300` / Phase T `#307` / `#308` / `#311`)

Desktop **Technical details → Peer dial** queues `run_opt_in_peer_dial` on an `async_jobs` dial slot (`#308` — not on the egui thread; trusted peer + **explicit** `dial_addr`):

1. Snapshot AddressBook, then trial-upsert the operator-supplied address
2. `aira_peer::dial` (hello + Noise XX)
3. On **failure**: restore the pre-attempt AddressBook (`#311` — known A survives failed candidate B)
4. On **success**: keep the candidate as dial authority; persist `peers/dial_session_evidence.json` (handshake hash + endpoint + timestamp)
5. Mesh projection may show **last confirmed handshake** while evidence is fresh — **`live_session_count` stays unobserved** (`None`); no reachability DIRECT

Setup / invite / Stop→Start alone still leave live sessions unobserved (`None`). Public bind is not the Desktop default.

Profile matrix and RFC index: [`desktop-network-profiles.md`](desktop-network-profiles.md).
RFC: [`AIRA-RFC-0187`](../specs/rfc/AIRA-RFC-0187-opt-in-peer-dial-session-evidence.md), honesty [`AIRA-RFC-0193`](../specs/rfc/AIRA-RFC-0193-dial-evidence-ne-live-session.md), off-UI [`AIRA-RFC-0194`](../specs/rfc/AIRA-RFC-0194-opt-in-dial-off-ui-thread.md), AddressBook candidate [`AIRA-RFC-0197`](../specs/rfc/AIRA-RFC-0197-addressbook-candidate-honesty.md).
