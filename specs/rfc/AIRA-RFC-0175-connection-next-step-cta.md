# AIRA-RFC-0175 — Connection next-step CTA (RFC-D)

## 1. Summary

Phase R `#287`: System → Connection exposes **exactly one** primary CTA from an honest state matrix (RestartNeeded → enable P1 → import invite → refresh reachability → none when Direct/Relayed). Refresh is no longer the only Unknown/LocalOnly action. RFC-0174 stays file-free until `#294`.

## 2. Problem Statement

Connection showed honesty copy and a Refresh button for Unknown/LocalOnly while invite / profile / Stop→Start lived under Technical details — explain ≫ instruct.

## 3. Motivation

`phase-r-plan` R1 / §9 and `desktop-ux` §4: each System section has at most one corrective action aligned with state, without fake CONNECTED/Applied.

## 4. Scope

- Pure `connection_cta::primary_connection_cta` matrix + unit tests
- `ui_sys_connection` renders that single primary button
- EN/UK CTA labels; Help routing per CTA
- Living smoke tip: `#287` DONE → first OPEN `#288`
- Analyze-323 living matrix

## 5. Non-Goals

```text
Promote invite/profile out of tech collapse (#288)
Human-primary raw-enum demotion (#289)
Help connect scenario rewrite (#291)
Consolidating RFC-0174 (#294)
```

## 6. Compatibility / Security

No Core/ledger changes. UNKNOWN≠OFFLINE and AddressBook≠sessions preserved. CTA never claims CONNECTED.

## 7. Rollout

QUEUE `#287` → Analyze-323 → PR; next `#288` promote connect controls.
