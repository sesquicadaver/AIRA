# AIRA-RFC-0182 — Phase S Cross-boundary Integrity (closure)

## 1. Summary

Consolidating documentation atom (`#305`): Phase S `#295`–`#305` complete — cross-boundary integrity & connect honesty after QUEUE R closed (@ RFC-0174). Reachability key-bound admission, model descriptor root-scoped verify, identity incomplete-pair fail-closed, Connect Help boundary, opt-in peer dial evidence, CTA Stop→Start, submit∥lifecycle admission, observe miss off-UI, and reachability durability honesty are each closed under RFC-0183…RFC-0191. Honesty invariants preserved (UNKNOWN≠OFFLINE; AddressBook≠sessions; no fake CONNECTED/Applied/DIRECT without evidence). C1 `Calculate 2 + 2` stays execution-basic. Anti-mission (GPU marketplace / LLM-in-Core) unchanged. `aira-core` gains no GUI/ledger implementation deps. Public bind / auto-trust remain non-default. **QUEUE S closed**; no OPEN S atoms. This atom does **not** invent Phase T.

## 5. Non-Goals

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
New Network tab / IA rewrite
Fake CONNECTED / Applied / DIRECT without evidence
Inventing Phase T / next feature phase in this atom
Rewriting Phase O–R history as never-DONE
Public bind / auto-trust as Desktop default
Joint-atomic reachability replay+state (explicit non-goal of #304)
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| S0 Wiring | `#295` | `phase-s-plan.md`; QUEUE; `phase_s_doc`; RFC-0182 reserved file-free |
| S1 Key-bound | `#296` | reachability key-bound admission (RFC-0183) |
| S1 Descriptor | `#297` | model descriptor root-scoped verify (RFC-0184) |
| S1 Identity | `#298` | incomplete-pair fail-closed (RFC-0185) |
| S1 Help boundary | `#299` | setup ≠ remote; loopback ≠ dial (RFC-0186) |
| S1 Opt-in dial | `#300` | `run_opt_in_peer_dial` + evidence (RFC-0187) |
| S2 CTA matrix | `#301` | Stop→Start (RFC-0188) |
| S2 Submit gate | `#302` | `admit_submit_lifecycle` (RFC-0189) |
| S2 Observe | `#303` | `OBSERVE_HASH_PENDING` / `sha256_path` (RFC-0190) |
| S2 Durability | `#304` | replay before state; no joint atomic (RFC-0191) |
| S Close | `#305` | this RFC; QUEUE S closed |

## 10. Per-atom contracts

```text
RFC-0183  Reachability key-bound admission (#296)
RFC-0184  Model descriptor root-scoped verify (#297)
RFC-0185  Identity incomplete-pair fail-closed (#298)
RFC-0186  Connect Help boundary honesty (#299)
RFC-0187  Opt-in peer dial / session evidence (#300)
RFC-0188  CTA Stop→Start matrix (#301)
RFC-0189  Submit∥Start/Stop admission (#302)
RFC-0190  Observe miss off UI-thread (#303)
RFC-0191  Reachability durability honesty (#304)
```

Id **confirmed free** 2026-09-09 (no `AIRA-RFC-0182*` in tree until this atom).

## 12. Status honesty

```text
Phase S (RFC-0182)     = cross-boundary integrity DONE
Reachability key-bound = same-ID / foreign-key reject (#296)
Descriptor reopen      = node-signed verify without process priming (#297)
Identity pair          = incomplete → fail-closed (#298)
Help connect           = setup ≠ remote; loopback ≠ dial (#299)
Opt-in dial            = explicit addr + dial evidence (#300)
CTA after Stop         = Start when next step (#301)
Submit∥lifecycle       = mutual exclusion (#302)
Observe miss           = background streaming hash (#303)
Durability             = replay≠state joint atomic honesty (#304)
UNKNOWN≠OFFLINE        = preserved
book≠sessions          = preserved
Global live mesh       = still PARTIAL @ N-fix (RFC-0139); unchanged by S
```

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_s_doc
cargo test -p aira-peer --lib reachability_state
cargo test -p aira-flow --lib activate_gate
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / execution-basic.
`aira-core` MUST NOT gain GUI / Help / egui / ledger implementation deps.
