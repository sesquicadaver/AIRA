# AIRA-RFC-0169 — Reachability NAT endpoints (RFC-D)

## 1. Summary

Phase Q `#280`: dial (advertised) endpoint and accept TCP `local_addr` may differ under NAT while remaining one challenge/session. Inbound local evidence no longer fail-closes on `bound_endpoint == challenge.endpoint`; binding is Noise handshake + transcript + INBOUND. Outbound probe attestation still requires dial endpoint ≡ challenge. RFC-0164 stays file-free until `#285`.

## 2. Problem Statement

`#270` required endpoint equality on both dial and accept. Accept binds `TcpStream::local_addr()` (often private under NAT) while the challenge carries the advertised/public endpoint the probe dialed — a valid session was rejected.

## 3. Motivation

`phase-q-plan` `#280` / acceptance: NAT dial≠local accepted endpoints can produce valid local evidence.

## 4. Scope

- Relax inbound `export_from_session` endpoint equality (`#280`)
- Keep outbound `issue_for_authenticated_session` strict
- NAT simulation test + updated `#270` mismatch expectations
- QUEUE → `#281`

## 5. Non-Goals

```text
Reachability evidence admission (#281) — DONE @ RFC-0170
Transcript domain / schema bump
CGNAT netns CI
```

## 6. Compatibility / Security

Wrong advertised endpoint still rejected on probe issue. Cross-session / forged transcript still fail `verify_with_local_evidence`.

## 7. Rollout

QUEUE `#280` → Analyze-315 → PR; next `#281` Reachability evidence admission.
