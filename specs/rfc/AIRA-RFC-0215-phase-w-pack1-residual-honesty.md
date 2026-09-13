# AIRA-RFC-0215 — Phase W Pack 1 residual honesty (closure)

## 1. Summary

Consolidating documentation atom (`#342`): Phase W `#331`–`#342` complete — Pack 1 residual honesty after QUEUE V closed (@ RFC-0208) and audit `d1115f2` Pack 1 PARTIAL (D1–D6). Math capsule fidelity, strict submit decoding, constraints enforce-or-reject, admission boundary verify, activate evidence authority, production activation trust, safe weights materialization, backend verified binding, reuse candidate independent check, and result/verify task binding are each closed under RFC-0216…RFC-0225. Honesty invariants preserved (choice never grants VERIFIED; mock remains explicit; C1 stays execution-basic; used-model requires verified binding). Anti-mission (GPU marketplace / LLM-in-Core) unchanged. `aira-core` gains no GUI/ledger/LLM runtime deps. Public bind / auto-trust remain non-default. Pack 2 multi-model GUI stays Phase X (`#343`–`#358`). **QUEUE W closed**; no OPEN W atoms. This atom does **not** invent Pack 3–7.

## 5. Non-Goals

```text
GPU marketplace / LLM-in-Core
Web/Tauri rewrite
Pack 2 Settings catalog / Work «Порівняти» GUI (Phase X)
Pack 3–7 repair packages
Rewriting Phase V as never-DONE / inventing Phase Y here
Public bind / auto-trust as Desktop default
Weakening observe / activate admission hash
Minting VERIFIED for generate-local
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| W0 Wiring | `#331` | `phase-w-plan.md`; QUEUE; `phase_w_doc`; RFC-0215 reserved file-free |
| W1 Math | `#332` | capsule fidelity; no default `2+2` (RFC-0216) |
| W1 Strict submit | `#333` | unknown nested fields → 4xx (RFC-0217) |
| W1 Constraints | `#334` | enforce-or-reject matrix (RFC-0218) |
| W1 Admission boundary | `#335` | text/hash/kind/schema (RFC-0219) |
| W1 Activate evidence | `#336` | pointer locator-only; primary evidence (RFC-0220) |
| W1 Production trust | `#337` | no implicit local-test (RFC-0221) |
| W1 Safe materialize | `#338` | no-follow + post-copy + bounded (RFC-0222) |
| W1 Backend binding | `#339` | mismatch reject; mock ≠ used-model (RFC-0223) |
| W1 Reuse candidate | `#340` | foreign VRA ≠ Completed (RFC-0224) |
| W1 Result/verify binding | `#341` | swap refs reject; generate honesty (RFC-0225) |
| W Close | `#342` | this RFC; QUEUE W closed; Pack 1 residual honesty DONE |

## 10. Per-atom contracts

```text
RFC-0216  Math capsule fidelity (#332)
RFC-0217  Strict submit decoding (#333)
RFC-0218  Constraints enforce-or-reject (#334)
RFC-0219  Admission boundary verify (#335)
RFC-0220  Activate evidence authority (#336)
RFC-0221  Production activation trust (#337)
RFC-0222  Safe weights materialization (#338)
RFC-0223  Backend verified binding (#339)
RFC-0224  Reuse candidate independent check (#340)
RFC-0225  Result/verify task binding (#341)
```

Id **confirmed free** 2026-09-11 (no `AIRA-RFC-0215*` in tree until this atom).

## 12. Status honesty

```text
Phase W (RFC-0215)     = Pack 1 residual honesty DONE
Math capsule           = expression fidelity; unsupported ≠ 2+2 (#332)
Strict submit          = unknown fields fail closed (#333)
Constraints            = enforce or explicit unsupported (#334)
Admission boundary     = text/hash/kind/schema (#335)
Activate evidence      = primary signed evidence authority (#336)
Production trust       = no implicit local-test (#337)
Safe materialize       = no-follow + post-copy + bounded (#338)
Backend binding        = mock ≠ used-model; mismatch reject (#339)
Reuse candidate        = index hit + independent check (#340)
Result/verify binding  = swap refs reject; generate ≠ false VF (#341)
C1 2+2                 = still VERIFIED via execution-basic
Pack 2 GUI             = Phase X / still separate
```

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_w_doc
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe` / execution-basic.
`aira-core` MUST NOT gain GUI / Help / egui / ledger / LLM implementation deps.

## 16. Handoff

```text
QUEUE W closed @ RFC-0215
  → first OPEN #343 (Phase X wiring; Pack 2 GUI)
  → RFC-0226 reserved file-free until #358
```
