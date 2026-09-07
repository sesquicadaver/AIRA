# AIRA-RFC-0152 — Desktop System status screen (Program / Model / Connection / Events)

## 1. Summary

Phase O `#261`: present System as four human-language sections — **Program**, **Model**, **Connection**, **Recent events** — with honesty texts (UNKNOWN ≠ OFFLINE; AddressBook ≠ live sessions; model not invented). Technical mesh/profile controls stay collapsed. RFC-0146 stays file-free until `#265`.

## 2. Problem Statement

System still exposed lifecycle + raw mesh/network ops as the primary body, fighting `desktop-ux` §4.

## 3. Motivation

Users need “is it working?” answers before operator controls.

## 4. Scope

- `system_view` conclusions + `SystemSnapshot` on status refresh
- UI sections with one corrective action each where applicable
- Model section: explicit NotChecked (no fake selected model)
- Connection: human banner + book vs live sessions honesty
- Events: last problem / discovery / invite / restart hint
- Tests; RFC-D; QUEUE → `#262`

## 5. Non-Goals

```text
Settings Saved≠Applied lifecycle (#262)
Offline F1 articles (#263/#264)
Live session feed inventing zeros
RFC-0146 consolidating body (#265)
```

## 6. Compatibility / Security

No Core/ledger changes. Projection remains read-only from existing stores.

## 7. Rollout

QUEUE `#261` → Analyze-296 → PR; next `#262` Settings apply.
