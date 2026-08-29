# AIRA-RFC-0079 — CRP local adapter (capability≠node)

## 1. Summary

Phase H `#166`: in-process `LocalCrpAdapter` in `aira-protocol` maps a CRP Route Request to Route Candidate(s) via local `DiscoveryRegistry`. Routing is by **Capability → provider CSU**; Node-keyed providers are rejected. Contract aligns with Book II §10 and schemas from `#165`.

## 5. Non-Goals

Multi-node CRP mesh; marketplace ranking; Policy Gate bind (`#168`); route events (`#169`); B2-006 C3 case (`#170`); status PARTIAL (`#171`).

## 10. Behavior

```text
route(request, discovery) → Candidates(chain) | Failure(reason)
MUST NOT bind provider with `:node:` in ref
MUST return capability_chain hops (capability_ref + provider_csu) or Failure
```

## 15. Tests

```text
cargo test -p aira-protocol crp_local
```
