# AIRA-RFC-0174 — Phase R Actionable Desktop Connection UX (closure)

## 1. Summary

Consolidating documentation atom (`#294`): Phase R `#286`–`#294` complete — actionable Desktop Connection UX after QUEUE Q closed (@ RFC-0164). Connection next-step CTA, promoted connect controls, human-primary mesh/strip language, actionable problem/strip, Help connect scenario, UK mesh/discovery parity, and cold-start empty-profile guidance are each closed under RFC-0175…RFC-0181. Honesty invariants preserved (UNKNOWN≠OFFLINE; AddressBook≠sessions; no fake CONNECTED/Applied). C1 `Calculate 2 + 2` stays execution-basic. Anti-mission (GPU marketplace / LLM-in-Core) unchanged. `aira-core` gains no GUI/ledger implementation deps. **QUEUE R closed**; no OPEN R atoms. This atom does **not** invent Phase S.

## 5. Non-Goals

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
New Network tab / IA rewrite
Fake CONNECTED / Applied without runtime
Inventing Phase S / next feature phase in this atom
Rewriting Phase O–Q history as never-DONE
Public bind / auto-trust as Desktop default
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| R0 Wiring | `#286` | `phase-r-plan.md`; QUEUE; `phase_r_doc`; RFC-0174 reserved file-free |
| R1 CTA | `#287` | Connection next-step CTA matrix (RFC-0175) |
| R2 Promote | `#288` | `ui_connect_primary` outside tech (RFC-0176) |
| R3 Language | `#289` | `mesh_language` human-primary (RFC-0177) |
| R4 Problem | `#290` | `problem_action` human next-step (RFC-0178) |
| R5 Help | `#291` | Help connect scenario (RFC-0179) |
| R6 UK | `#292` | UK mesh/discovery parity (RFC-0180) |
| R7 Cold-start | `#293` | empty-profile guidance (RFC-0181) |
| R Close | `#294` | this RFC; QUEUE R closed |

## 10. Per-atom contracts

```text
RFC-0175  Connection next-step CTA (#287)
RFC-0176  Promote connect controls (#288)
RFC-0177  Human-primary mesh/strip (#289)
RFC-0178  Actionable problem/strip (#290)
RFC-0179  Help connect scenario (#291)
RFC-0180  UK mesh/discovery parity (#292)
RFC-0181  Cold-start empty profile (#293)
```

Id **confirmed free** 2026-09-09 (no `AIRA-RFC-0174*` in tree until this atom).

## 12. Status honesty

```text
Phase R (RFC-0174)     = actionable Connection UX DONE
Primary CTA            = one next-step on Connection (#287)
Connect controls       = invite/P0–P2 outside Technical details (#288)
Mesh/strip language    = human conclusion > raw enum (#289)
Problem/strip          = human next-step; wire secondary (#290)
Help connect           = invite→profile→restart scenario (#291)
UK mesh/discovery      = chrome parity; CLI tech-only (#292)
Cold-start             = P0+empty book guidance; no fake Connected (#293)
UNKNOWN≠OFFLINE        = preserved
book≠sessions          = preserved
Global live mesh       = still PARTIAL @ N-fix (RFC-0139); unchanged by R
```

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_r_doc
cargo test -p aira-desktop connection_cta
cargo test -p aira-desktop cold_start
cargo clippy -p aira-desktop -- -D warnings
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / execution-basic.
`aira-core` MUST NOT gain GUI / Help / egui / ledger implementation deps.
