# AIRA-RFC-0104 — Phase K Local LLM Execution CSU closure

## 1. Summary

Consolidating documentation atom (`#216`): Phase K `#209`–`#216` complete — local LLM Execution CSU documented in `README.md`, `docs/implementation-status.md`, `docs/phase-k-plan.md`, Desktop Work generate path, and this RFC. Work tab / `POST /v1/problems` runs non-math prompts through Reduction (`#212`) + plane register (`#213`) + MockBackend default; human-first UX leads with `result.result` + status, not raw VRA JSON. C1 `Calculate 2 + 2` stays `math.eval.safe` / execution-basic. Activate gate (`#214`) remains fail-closed without Phase D activate (Desktop does not fake VERIFIED). CI keeps MockBackend (no ollama). Anti-mission (GPU marketplace / LLM-in-Core / blockchain) unchanged. QUEUE K closed; no OPEN K atoms. A next phase is not activated here.

## 5. Non-Goals

GPU marketplace; LLM runtime in `aira-core` (inference host); blockchain / scheduler in Core; requiring process backend in CI; llama/ggml in Core; full semantic verify of free-form LLM output; inventing Phase L.

## 7. Deliverables (rollup)

| Band | QUEUE | Scope |
|------|-------|--------|
| K0 govern | `#209` | `phase-k-plan.md`; living `phase_k_doc` |
| K1 capsule | `#210` | `aira:schema:execution:generate-local:0.1` (RFC-0105) |
| K2 CSU mock | `#211` | `execution-llm` + MockBackend (RFC-0106) |
| K3–K4 bind | `#212`–`#213` | Reduction generate-local; plane register (RFC-0107, RFC-0108) |
| K5 activate | `#214` | fail-closed without Phase D activate (RFC-0109) |
| K6 process | `#215` | argv CLI adapter; CI mock (RFC-0110) |
| K7 Desktop | `#216` | Work generate path; this RFC; QUEUE K closed |

## 10. Per-atom contracts

```text
RFC-0105  generate-local payload schema (#210)
RFC-0106  execution-llm MockBackend (#211)
RFC-0107  Reduction generate-local bind (#212)
RFC-0108  plane register execution-llm (#213)
RFC-0109  activate gate (#214)
RFC-0110  process backend (#215)
```

Id **confirmed free** at `#209` (no `AIRA-RFC-0104*` in tree until this atom).

Desktop Work does **not** add a new HTTP route. Generate uses the existing `POST /v1/problems` → `LocalSession` → `OperationalPlane` path. Status `executed` is CapsuleCompleted, not VERIFIED.

## 15. Tests

```text
cargo test -p aira-desktop-runtime --test phase_k_doc
cargo test -p aira-desktop
cargo test -p aira-conformance --lib
cargo test -p aira-flow --lib calculate_two_plus_two
```
