# AIRA-RFC-0177 — Human-primary mesh/strip language (RFC-D)

## 1. Summary

Phase R `#289`: status strip Network cell and mesh tech banner use the same `ConnectionConclusion` mapping as System → Connection. Raw top-level enums (`UNKNOWN`, `LOCAL ONLY`, …) are secondary/monospace. Unexpected labels no longer paint strip as offline. RFC-0174 stays file-free until `#294`.

## 2. Problem Statement

Strip matched raw strings with a catch-all → offline, while Connection mapped unknown labels to Unknown — contradictory primary language. Mesh tech showed raw `UNKNOWN` as the strong banner beside human copy.

## 3. Motivation

`phase-r-plan` R3 / `desktop-ux` §2.3: human conclusion primary; wire enums secondary; strip must not contradict Connection.

## 4. Scope

- `mesh_language::strip_network_phrase` / `strip_network_from_top_level`
- `ui_status_strip` uses shared mapping
- `ui_mesh_status`: human `sys_conn_*` primary; raw `top_level` monospace secondary
- Living smoke tip `#289` DONE → `#290`; Analyze-325; RFC-0177

## 5. Non-Goals

```text
Actionable problem footer (#290)
Help connect scenario (#291)
UK mesh/discovery i18n parity (#292)
Consolidating RFC-0174 (#294)
```

## 6. Compatibility / Security

No Core changes. UNKNOWN≠OFFLINE preserved; Direct/Relayed still require Current quality for «connected».

## 7. Rollout

QUEUE `#289` → Analyze-325 → PR; next `#290` actionable problem/strip.
