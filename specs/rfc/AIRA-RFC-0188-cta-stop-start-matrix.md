# AIRA-RFC-0188 — CTA Stop→Start matrix (RFC-D)

## 1. Summary

Phase S `#301`: Connection primary CTA after Stop on a peer-capable profile offers **Start**, not Refresh-only, without inventing CONNECTED. RFC-0182 stays file-free until `#305`.

## 2. Problem Statement

After Stop, mesh often stays Unknown/LocalOnly while the node is down. The Phase R matrix could still pick RefreshStatus, leaving a dead-end instead of Start to re-apply/observe.

## 3. Motivation

`phase-s-plan` `#301` / post-R audit §6: stopped runtime → Start when that is the next honest step.

## 4. Scope

- `primary_connection_cta` / `ConnectionPrimaryCta::StartToApply`: Stopped/Failed + `requires_peer_listen` → Start (after RestartNeeded / P0 / empty-book priority)
- Transition unit tests in `connection_cta.rs`
- QUEUE tip → `#302`
- Docs / `phase_s_doc` living smoke

Does **not** invent CONNECTED; Refresh remains valid while Running with Unknown.

## 5. Non-Goals

```text
Submit∥lifecycle admission (#302)
Observe miss off UI-thread (#303)
Inventing CONNECTED / DIRECT from Start alone
Changing Help-boundary (#299) or dial (#300)
```

## 6. Compatibility / Security

CTA selection only. Lifecycle Start still uses existing supervised start path.

## 7. Rollout

QUEUE `#301` → Analyze-337 → PR; next `#302` Submit∥Start/Stop admission.
