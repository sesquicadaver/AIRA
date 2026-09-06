# AIRA-RFC-0142 — Reachability session / transcript bind (RFC-D)

## 1. Summary

Phase N-fix `#250`: successful `ReachabilityAttestation` must bind to an authenticated Noise XX session (handshake hash + canonical session transcript). Applying `DIRECT_REACHABLE` requires the target's **local inbound** transcript to match the attestation. A signed claim without connect cannot set DIRECT. RFC-0139 stays file-free until `#254`.

## 2. Problem Statement

Post-N audit: attestation could be issued as a signed claim without proving an inbound path. That over-claims `DIRECT_REACHABLE`.

## 3. Motivation

Probe must dial; both ends share the Noise handshake hash; transcript binds `challenge_id|nonce|target|probe|hs`. Target applies DIRECT only when local inbound transcript matches.

## 4. Scope

- `noise_xx_*` expose handshake hash; `AuthenticatedPeer` stores it
- `session_transcript_hex` / `REACHABILITY_SESSION_DOMAIN`
- `issue_for_authenticated_session`; `success=true` via `issue_for_challenge` rejected
- `ReachabilityResult::verify_with_local_session`; `apply_successful_probe(..., local_tx)`
- CLI `--session-transcript` with `--result-json`
- Tests: session-bound probe → DIRECT; wrong/missing transcript fails; no-connect issue fails
- RFC-D this file; QUEUE → `#251`

## 5. Non-Goals

```text
Expiry before promote (#251)
netns NAT/firewall (#252)
next_candidate_port wrap (#253)
RFC-0139 consolidating body (#254)
```

## 6. Compatibility / Security

Additive attestation fields (default empty). Hairpin still forbidden. `DISCOVERED ≠ TRUSTED`. No ledger deps in `aira-core`.

## 7. Rollout

QUEUE `#250` → Analyze-285 → PR; next `#251` expiry before promote.
