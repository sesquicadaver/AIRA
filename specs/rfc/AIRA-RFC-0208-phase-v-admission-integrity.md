# AIRA-RFC-0208 — Phase V Admission integrity / Repair Pack 1 (closure)

## 1. Summary

Consolidating documentation atom (`#330`): Phase V `#323`–`#330` complete — Repair Pack 1 admission integrity after QUEUE U closed (@ RFC-0198) and Pack 0 DONE (@ RFC-0207). Immutable admission snapshot, submit constraints API, reuse-after-constraints, activate_verified hash continuity, Capsule↔Output↔Result executor facts, and Pack 1 §6 fail-closed e2e are each closed under RFC-0209…RFC-0214. Honesty invariants preserved (choice never grants VERIFIED; mock remains explicit; C1 stays execution-basic). Anti-mission (GPU marketplace / LLM-in-Core) unchanged. `aira-core` gains no GUI/ledger/LLM runtime deps. Public bind / auto-trust remain non-default. Pack 2 multi-model GUI stays separate. **QUEUE V closed**; no OPEN V atoms. This atom does **not** invent a next feature phase.

## 5. Non-Goals

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Pack 2 Settings catalog / Work «Порівняти» GUI
Pack 3–7 repair packages
Inventing Phase W / next feature phase in this atom
Rewriting Phase O–U history as never-DONE
Public bind / auto-trust as Desktop default
Weakening observe / activate admission hash
Minting VERIFIED for generate-local
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| V0 Wiring | `#323` | `phase-v-plan.md`; QUEUE; `phase_v_doc`; RFC-0208 reserved file-free |
| V1 Snapshot | `#324` | AdmissionSnapshot + persist-on-admit (RFC-0209) |
| V1 Submit API | `#325` | AdmissionConstraints on HTTP/CLI/Desktop (RFC-0210) |
| V1 Reuse | `#326` | reuse key ⊇ snapshot; RequireNewExecution (RFC-0211) |
| V1 Activate | `#327` | verify→activate content hash (RFC-0212) |
| V1 Binding | `#328` | executor facts + CapsuleCompleted (RFC-0213) |
| V2 E2E | `#329` | Pack 1 §6 fail-closed suite (RFC-0214) |
| V Close | `#330` | this RFC; QUEUE V closed; repair-package-1 DONE |

## 10. Per-atom contracts

```text
RFC-0209  Admission snapshot (#324)
RFC-0210  Submit API constraints (#325)
RFC-0211  Reuse after constraints (#326)
RFC-0212  activate_verified hash continuity (#327)
RFC-0213  Capsule↔Output↔Result binding (#328)
RFC-0214  E2E fail-closed Pack 1 subset (#329)
```

Id **confirmed free** 2026-09-10 (no `AIRA-RFC-0208*` in tree until this atom).

## 12. Status honesty

```text
Phase V (RFC-0208)     = admission integrity / Repair Pack 1 DONE
Admission snapshot     = immutable on admit (#324–#325)
Settings mid-run       ≠ mutate admitted task (#325 / #329)
Reuse after constraints= model/context key; require-new (#326)
activate_verified      = post-copy == VerifiedPointer.content_hash (#327)
Executor facts         = model_ref + model_content_hash stamped (#328)
Pack 1 §6 e2e          = living fail-closed suite (#329)
C1 2+2                 = still VERIFIED via execution-basic
Pack 2 GUI             = still separate / not DONE
```

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_v_doc
cargo test -p aira-flow --test phase_v_pack1_e2e
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / execution-basic.
`aira-core` MUST NOT gain GUI / Help / egui / ledger / LLM implementation deps.
