# AIRA-RFC-0192 — Phase T Operation lifecycle honesty (closure)

## 1. Summary

Consolidating documentation atom (`#312`): Phase T `#306`–`#312` complete — operation lifecycle honesty after QUEUE S closed (@ RFC-0182). Dial evidence ≠ live session, opt-in dial off UI-thread, observe miss fail durable, Quit∥Submit completion, and AddressBook candidate honesty are each closed under RFC-0193…RFC-0197. Honesty invariants preserved (UNKNOWN≠OFFLINE; AddressBook≠sessions; dial history ≠ live count; no silent AddressBook overwrite on failed probe). C1 `Calculate 2 + 2` stays execution-basic. Anti-mission (GPU marketplace / LLM-in-Core) unchanged. `aira-core` gains no GUI/ledger implementation deps. Public bind / auto-trust remain non-default. **QUEUE T closed**; no OPEN T atoms. This atom does **not** invent a next feature phase.

## 5. Non-Goals

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
New Network tab / IA rewrite
Fake CONNECTED / Applied / DIRECT / live_session_count without evidence
Inventing Phase U / next feature phase in this atom
Rewriting Phase O–S history as never-DONE
Public bind / auto-trust as Desktop default
Weakening observe admission hash
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| T0 Wiring | `#306` | `phase-t-plan.md`; QUEUE; `phase_t_doc`; RFC-0192 reserved file-free |
| T1 Dial evidence | `#307` | evidence ≠ live_session_count (RFC-0193) |
| T1 Dial off-UI | `#308` | jobs/slot; no UI `block_on` (RFC-0194) |
| T1 Observe fail | `#309` | Pending/Ready/Failed; no rehash storm (RFC-0195) |
| T2 Quit∥Submit | `#310` | deferred Stop→Close after submit (RFC-0196) |
| T2 AddressBook | `#311` | trial upsert; rollback on fail (RFC-0197) |
| T Close | `#312` | this RFC; QUEUE T closed |

## 10. Per-atom contracts

```text
RFC-0193  Dial evidence ≠ live session (#307)
RFC-0194  Opt-in dial off UI-thread (#308)
RFC-0195  Observe miss fail durable (#309)
RFC-0196  Quit∥Submit completion (#310)
RFC-0197  Dial AddressBook candidate honesty (#311)
```

Id **confirmed free** 2026-09-09 (no `AIRA-RFC-0192*` in tree until this atom).

## 12. Status honesty

```text
Phase T (RFC-0192)     = operation lifecycle honesty DONE
Dial evidence          = history ≠ live_session_count (#307)
Dial off-UI            = jobs/slot; F1/nav during dial (#308)
Observe fail           = sticky Failed; no rehash storm (#309)
Quit∥Submit            = deferred Stop→Close; explicit wait (#310)
AddressBook candidate  = rollback on failed dial (#311)
UNKNOWN≠OFFLINE        = preserved
book≠sessions          = preserved
Global live mesh       = still PARTIAL @ N-fix (RFC-0139); unchanged by T
```

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_t_doc
cargo test -p aira-desktop-runtime --lib peer_dial
cargo test -p aira-flow --lib activate_gate
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / execution-basic.
`aira-core` MUST NOT gain GUI / Help / egui / ledger implementation deps.
