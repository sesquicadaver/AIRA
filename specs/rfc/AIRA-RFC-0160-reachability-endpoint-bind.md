# AIRA-RFC-0160 — Reachability endpoint + direction bind (RFC-D)

## 1. Summary

Phase P `#270`: successful DIRECT proof binds the Noise session transcript to the **challenge endpoint** and **INBOUND** proof direction. Probe attestation requires an **OUTBOUND** dial whose socket matches that endpoint. Applying DIRECT requires signed [`ReachabilityLocalEvidence`] from the target's accept session — a bare CLI `--session-transcript` string is rejected. RFC-0156 stays file-free until `#274`.

## 2. Problem Statement

After `#250`, transcript bound identities + Noise hash but not endpoint/direction. Caller-supplied `observed_endpoint` and CLI transcript strings could claim DIRECT for endpoint Y using a session on X.

## 3. Motivation

`phase-p-plan` / post-O audit §8: attestation must bind endpoint + session direction; CLI transcript-alone ≠ locally verified.

## 4. Scope

- `SessionDirection::{Inbound,Outbound}`; `bound_endpoint` on `AuthenticatedPeer`
- Transcript domain `aira:reachability:session:v2` includes endpoint + `INBOUND`
- `ReachabilityLocalEvidence` (signed); `verify_with_local_evidence`; `apply_successful_probe(..., evidence)`
- CLI `--session-evidence`; reject bare `--session-transcript`
- Tests: happy path; wrong-endpoint reject; forged evidence reject
- QUEUE → `#271`

## 5. Non-Goals

```text
EVM/JSON-RPC honesty (#271)
Lifecycle non-blocking (#272)
Consolidating RFC-0156 (#274)
```

## 6. Compatibility / Security

Transcript domain bumped to v2 (old v1 attestations will not verify). Hairpin still forbidden. No Core/ledger deps.

## 7. Rollout

QUEUE `#270` → Analyze-305 → PR; next `#271` EVM/JSON-RPC honesty.
