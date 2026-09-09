# AIRA-RFC-0186 — Connect Help boundary honesty (RFC-D)

## 1. Summary

Phase S `#299`: Help EN+UK and Connection CTA copy state the connect **boundary** explicitly — setup ≠ remote session; loopback/listen ≠ peer dial address — without implementing dial (`#300`). RFC-0182 stays file-free until `#305`.

## 2. Problem Statement

Phase R connect Help documented invite → profile → Stop/Start as the happy path. Users could read that as “remote peer session complete,” especially when Desktop listen stays loopback-bound and live sessions are unobserved.

## 3. Motivation

`phase-s-plan` `#299` / post-R audit §5: document honesty before any opt-in dial path.

## 4. Scope

- `docs/help/{en,uk}/network.connect.md` Boundary / Межа sections
- Cross-refs from trust + reachability Help
- Desktop `conn_boundary_guidance` (EN+UK) always on Connection; cold-start line softens expectations
- Living smoke: Help needles + `phase_s_doc` tip → `#300`
- QUEUE → `#300`

## 5. Non-Goals

```text
Opt-in peer dial / session evidence (#300)
Public bind / auto-trust as Desktop default
Inventing CONNECTED from setup alone
CTA Stop→Start matrix (#301)
```

## 6. Compatibility / Security

Docs and UI copy only. No network admission or dial behavior change.

## 7. Rollout

QUEUE `#299` → Analyze-335 → PR; next `#300` Opt-in peer dial / session evidence.
