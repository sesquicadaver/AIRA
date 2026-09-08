# AIRA-RFC-0176 — Promote connect controls (RFC-D)

## 1. Summary

Phase R `#288`: System → Connection shows **P0–P2 profile**, peer listen, and **friend invite/QR** outside Technical details. Advanced (P3/P4, federation, discovery) and raw mesh banner stay collapsed. RFC-0174 stays file-free until `#294`.

## 2. Problem Statement

After `#287` CTA, the invite → profile path was still buried under `sys_tech_details`, so the next-step button pointed at actions the user could not see.

## 3. Motivation

`phase-r-plan` R2: canonical connect scenario visible without mandatory tech collapse; honesty/advanced remain secondary.

## 4. Scope

- `ui_connect_primary` before Connection tech collapse
- `ui_network_advanced` + mesh under tech
- Living smoke tip `#288` DONE → `#289`; Analyze-324; RFC-0176

## 5. Non-Goals

```text
Human-primary raw-enum demotion (#289)
Help connect scenario rewrite (#291)
Consolidating RFC-0174 (#294)
```

## 6. Compatibility / Security

No Core changes. Same invite/profile actions as before; only layout priority changes.

## 7. Rollout

QUEUE `#288` → Analyze-324 → PR; next `#289` human-primary mesh/strip.
