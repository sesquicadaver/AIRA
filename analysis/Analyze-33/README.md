# Analyze-33 — Peer CLI Ops

**Scope:** `aira peer add|list|listen|dial|send` over Analyze-32 `aira-peer` (no Noise).

**Status:** ralplan → ralph → code-review **APPROVE/CLEAR**

## Ralplan (APPROVED — consensus)

### Principles
1. Operator path for P0 without changing hello/frame wire
2. Fail closed: address book add requires peer in `trust.json` (not revoked)
3. Loopback listen by default
4. Do not edit `Manifesto etc/**` or `Meditation_About/**`

### ADR
- **Decision:** CLI `Peer` subcommands wrapping `AddressBook` + `dial`/`listen`/`accept` + envelope ping
- **Why:** Makes decentralized peer links demo/operable in one cycle
- **Reject:** Noise XX, trust-delta, daemon multi-accept
- **Follow-up:** Noise under same CLI surface

### Acceptance
- add/list address book with trust gate
- listen one accept + recv envelope
- dial hello OK
- send dial+signed peer.ping; listen receives
- CLI smoke + clippy PASS; docs updated
