# AIRA-RFC-0193 — Dial evidence ≠ live session (RFC-D)

## 1. Summary

Phase T `#307`: opt-in dial evidence is **last confirmed handshake history**, not a live TCP session. Mesh projection keeps `live_session_count = None` after a successful dial that closes the socket; `last_confirmed_handshake` may still show while evidence is fresh. RFC-0192 stays file-free until `#312`.

## 2. Problem Statement

`#300` / RFC-0187 projected fresh `DialSessionEvidence` as `live_session_count = Some(1)` even though `run_opt_in_peer_dial` drops the session after capture. That blurred AddressBook ≠ live sessions and survived Desktop reopen for up to five minutes.

## 3. Motivation

`phase-t-plan` `#307` / post-S audit §2: last successful check ≠ current connection.

## 4. Scope

- `network_mesh`: fresh dial → handshake summary only; live count stays `None`
- Tests: flip `Some(1)` expectation; unit test history-without-live
- Help EN+UK + `desktop-peer.md` wording
- QUEUE tip → `#308`

## 5. Non-Goals

```text
Opt-in dial off UI-thread (#308)
Observe miss fail durable (#309)
Holding a long-lived GUI-owned peer session
Reachability DIRECT from dial
```

## 6. Compatibility / Security

No change to dial crypto or trust gates. Freshness window still limits how long history is shown.

## 7. Rollout

QUEUE `#307` → Analyze-343 → PR; next `#308` Opt-in dial off UI-thread.
