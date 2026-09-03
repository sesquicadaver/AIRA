# AIRA-RFC-0111 — Phase L execution-boundary hardening closure

## 1. Summary

Consolidating documentation atom (`#223`): Phase L `#217`–`#223` complete — AIRA → local LLM process boundary is fail-closed and honest. Activate is Phase D evidence/hash (RFC-0112), not pointer-exists. Child env is a whitelist (RFC-0113). Pipes are bounded during read (RFC-0114). ProblemRecord does not label executed as verified (RFC-0115). `network=none` is AIRA-mediated (RFC-0116): the generate adapter opens no sockets; the child is not an OS network-off sandbox. C1 `Calculate 2 + 2` stays `math.eval.safe` / execution-basic. CI keeps MockBackend (no ollama). Anti-mission (GPU marketplace / LLM-in-Core / blockchain) unchanged. QUEUE L closed; no OPEN L atoms. A next phase is not activated here. Inventing Phase M is a Non-Goal.

## 5. Non-Goals

```text
GPU marketplace
LLM-in-Core / LLM runtime in aira-core (inference host)
blockchain / scheduler in Core
Landlock / seccomp / Linux network namespace
requiring ollama/llama.cpp in CI
llama/ggml in aira-core
full semantic verify of free-form LLM output
inventing Phase M
```

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| L0 govern | `#217` | `phase-l-plan.md`; living `phase_l_doc` |
| L1 activate evidence | `#218` | hash + cache bytes + evidence (RFC-0112) |
| L2 child env | `#219` | `env_clear` + PATH/HOME/LANG (RFC-0113) |
| L3 bounded pipes | `#220` | cap during read; overflow CapsuleFailed (RFC-0114) |
| L4 ProblemRecord | `#221` | `execution_artifact_id`; GET honesty (RFC-0115) |
| L5 network=none | `#222` | AIRA-mediated none; not OS sandbox (RFC-0116) |
| L6 close | `#223` | this RFC; QUEUE L closed |

## 10. Per-atom contracts

```text
RFC-0112  activate evidence gate (#218)
RFC-0113  child env whitelist (#219)
RFC-0114  bounded pipes (#220)
RFC-0115  ProblemRecord split (#221)
RFC-0116  network=none honesty (#222)
```

Id **confirmed free** at `#217` (no `AIRA-RFC-0111*` in tree until this atom).

`constraints.network = none` remains AIRA-mediated. ollama-style child loopback is an explicit host-process exception, not OS `network=none` enforcement.

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_l_doc
cargo test -p aira-flow --lib -- calculate_two_plus_two_stays
cargo test -p aira-conformance --lib
```

C1 `Calculate 2 + 2` MUST stay `math.eval.safe`.
