# AIRA-RFC-0187 — Opt-in peer dial / session evidence (RFC-D)

## 1. Summary

Phase S `#300`: Desktop path offers opt-in dial of a **trusted** peer at an **explicit** address, persists Noise handshake evidence, and may project a fresh live-session observation — without public-bind default and without inventing DIRECT reachability from setup alone. RFC-0182 stays file-free until `#305`.

## 2. Problem Statement

After `#299`, Help correctly states setup ≠ remote session. Operators still needed a documented Desktop path to perform an authenticated dial and retain evidence that a handshake completed.

## 3. Motivation

`phase-s-plan` `#300` / post-R audit §5: explicit allowed address + dial + confirmed handshake evidence.

## 4. Scope

- `run_opt_in_peer_dial` in `aira-desktop-runtime` (`peer_dial.rs`)
- Durable `peers/dial_session_evidence.json`
- Mesh: fresh evidence → `live_session_count=Some(1)` + summary; no `apply_successful_probe`
- Desktop Technical-details UI (peer_id + dial_addr + Dial)
- Help EN+UK + `desktop-peer.md`
- Tests: fail-closed empty/untrusted; dual-root dial evidence without DIRECT
- QUEUE → `#301`

## 5. Non-Goals

```text
CTA Stop→Start matrix (#301)
Submit∥lifecycle (#302)
Public bind / auto-trust as Desktop default
Reachability DIRECT apply from dial alone
Holding a long-lived GUI-owned peer daemon session
```

## 6. Compatibility / Security

Requires prior trust admission. Explicit address upsert is opt-in. Desktop listen remains loopback-supervised.

## 7. Rollout

QUEUE `#300` → Analyze-336 → PR; next `#301` CTA Stop→Start matrix.
