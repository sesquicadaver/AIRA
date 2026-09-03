# AIRA-RFC-0117 — Phase M OS child sandbox closure

## 1. Summary

Consolidating documentation atom (`#230`): Phase M `#224`–`#230` complete — Linux OS isolation for the `ProcessBackend` child is opt-in and fail-closed. Landlock FS (RFC-0118), seccomp (RFC-0119), netns (RFC-0120), and sandbox-required (RFC-0121) are operator env / `ProcessBackend` hooks. Schema/docs distinguish OS isolation from RFC-0116 AIRA-mediated `constraints.network=none` (RFC-0122): adapter none is payload; OS layers are **not** encoded in generate-local. C1 `Calculate 2 + 2` stays `math.eval.safe` / execution-basic. CI keeps MockBackend (no ollama). Anti-mission (GPU marketplace / LLM-in-Core / blockchain) unchanged. QUEUE M closed; no OPEN M atoms. A next phase is not activated here. Inventing Phase N is a Non-Goal.

## 5. Non-Goals

```text
GPU marketplace
LLM-in-Core / LLM runtime in aira-core (inference host)
blockchain / scheduler in Core
macOS seatbelt / Windows job objects
WASM/hostile sandbox for weights
forcing ollama through netns
requiring ollama/llama.cpp in CI
llama/ggml in aira-core
full semantic verify of free-form LLM output
inventing Phase N
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| M0 govern | `#224` | `phase-m-plan.md`; living `phase_m_doc` |
| M1 Landlock FS | `#225` | Linux Landlock on child; fail-closed (RFC-0118) |
| M2 seccomp | `#226` | syscall filter; forbidden syscall CapsuleFailed (RFC-0119) |
| M3 netns | `#227` | CLONE_NEWNET for offline argv; ollama loopback fail-closed (RFC-0120) |
| M4 sandbox-required | `#228` | missing kernel / non-Linux / loopback fail-closed (RFC-0121) |
| M5 OS vs AIRA-mediated | `#229` | schema/docs honesty (RFC-0122) |
| M6 close | `#230` | this RFC; QUEUE M closed |

## 10. Per-atom contracts

```text
RFC-0118  Landlock FS (#225)
RFC-0119  seccomp (#226)
RFC-0120  netns (#227)
RFC-0121  sandbox-required (#228)
RFC-0122  OS isolation vs AIRA-mediated none (#229)
```

Id **confirmed free** at `#224` (no `AIRA-RFC-0117*` in tree until this atom).

`constraints.network = none` remains AIRA-mediated (RFC-0116). Optional OS layers (RFC-0118–0121) are operator opt-in, not payload fields.

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_m_doc
cargo test -p aira-flow --lib -- calculate_two_plus_two_stays
cargo test -p aira-conformance --lib
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.
