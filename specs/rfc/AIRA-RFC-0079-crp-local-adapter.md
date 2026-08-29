# AIRA-RFC-0079 — CRP local adapter (capability≠node)

## 1. Summary

Phase H `#166`–`#169`: in-process `LocalCrpAdapter` in `aira-protocol` maps a CRP Route Request to Route Candidate(s) via local `DiscoveryRegistry`. Routing is by **Capability → provider CSU**; Node-keyed providers are rejected. Multiple equivalent candidates when Discovery has multiple providers. Binding requires Policy Gate **ALLOW** on action `crp.bind` (`#168`). Optional `EventSink` emits `RouteSelected` / `RouteRejected` / `RouteFailed` (`#169`). Contract aligns with Book II §10 and schemas from `#165`.

## 5. Non-Goals

Multi-node CRP mesh; marketplace ranking; B2-006 C3 case (`#170`); status PARTIAL (`#171`).

## 10. Behavior

```text
route(request, discovery, events?) → Candidates(chains…) | Failure(reason)
  Failure → RouteFailed (if events)
bind(candidate, policy_gate, events?) → Bound | Denied
  ALLOW → Bound + RouteSelected; DENY/REQUIRE → Denied + RouteRejected
MUST NOT bind provider with `:node:` in ref
MUST support ≥2 equivalent candidates when multiple CSU providers exist
```

## 15. Tests

```text
cargo test -p aira-protocol crp_
```
