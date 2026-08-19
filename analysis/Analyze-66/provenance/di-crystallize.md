# DI crystallize — Analyze-66 / QUEUE #31

## In scope (Option A + A1 + B1 + C1 + D1 + E1 + F1)
1. STUN **Binding** client (RFC 5389), parse **XOR-MAPPED-ADDRESS** → reflexive `IP:port`.
2. CLI `peer stun query --stun-server host:port` (or env `AIRA_STUN_SERVER`); no default public STUN.
3. On success: print addr + write `peers/stun_reflexive.json` (`addr`, `stun_server`, `observed_at`).
4. `peer dht announce --from-stun`: addr from that file; with explicit `--addr` → fail-closed.
5. `dial` unchanged (book → TCP); no STUN-per-dial.
6. In-process UDP mock STUN + unit/integration tests (IPv4); no live external STUN in CI.
7. Docs: `docs/peer-link.md` dial path (stun query → reflexive file → dht announce → remote book → dial).

## Out
Full ICE / connectivity-check; UDP peer sessions; discv5; STUN on every dial; default public STUN; upsert into `address_book`; TURN; Hello/Noise/trust changes.

## Decision boundaries (agent-owned)
CLI names above; XOR-MAPPED; IPv4 mock; crate module under `aira-peer`.
