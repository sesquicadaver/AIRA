# AIRA-RFC-0178 — Actionable problem/strip next step (RFC-D)

## 1. Summary

Phase R `#290`: problem footer, System → Events, and the Work strip cell show a **human next-step** when a corrective `ActionId` exists. Stable `help:` / `try:` / error-code wires remain secondary. RFC-0174 stays file-free until `#294`.

## 2. Problem Statement

Problem surfaces led with wire ids (`help:node.lifecycle`, `try:node.start`) as the only next-action copy, so users saw operator keys instead of «press Start».

## 3. Motivation

`phase-r-plan` R4 / `desktop-ux`: user-facing next action; wire ids are not the sole copy.

## 4. Scope

- `problem_action::{action_next_step, action_strip_hint, ProblemActionView, strip_work_from_problem}`
- Problem footer + Events + status strip Work cell
- Living smoke tip `#290` DONE → `#291`; Analyze-326; RFC-0178

## 5. Non-Goals

```text
Help connect scenario rewrite (#291)
UK mesh/discovery parity (#292)
Clickable corrective buttons (optional later)
Consolidating RFC-0174 (#294)
```

## 6. Compatibility / Security

No Core changes. Corrective mapping unchanged (`ErrorCode::corrective_action`).

## 7. Rollout

QUEUE `#290` → Analyze-326 → PR; next `#291` Help connect scenario.
