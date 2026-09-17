# AIRA-RFC-0242 — Host LLM required; OP-001/C1 math legacy

## 1. Summary

Desktop Work and normative C0/C1/alpha acceptance **require a real host process LLM path**. OP-001 / C1 `Calculate 2 + 2` → VERIFIED 4.0 is **legacy non-normative** (historical example, not product acceptance). Reference MockBackend is not product Work. **Executed ≠ VERIFIED.**

## 2. Problem Statement

Product acceptance and Desktop hints treated deterministic `math.eval.safe` / `execution-basic` as the working path, while generate used mock. Operators need ≥1 deployed host LLM; calculator demo must not be the escape hatch.

## 3. Motivation

Authorized remapping: OP-001/C1-math is a legacy example, not the norm. Work fail-closes without Settings process + `ollama list` model. CI proves process executor smoke and VRA schema fixtures without mandatory `result=4`.

## 4. Scope

- `evaluate_work_readiness`: host LLM gate (`llm_backend=process` + `llm_ollama_model`); no math-without-model ready path
- Desktop i18n / Work hints: remove Calculate 2+2 as product example; mock banner = bind host LLM
- C1: `c1.pipeline.process_executor_executed` (`echo` ProcessBackend → Executed); VRA completeness from fixtures
- C0 causality + alpha DoD: process smoke instead of Calculate 2+2 VERIFIED
- `csu/execution-basic`: remains in-tree as **LEGACY** CSU (manifest still validates)

## 5. Non-Goals

```text
Deleting execution-basic crate this pass
Forging VERIFIED for ollama / AlwaysActivated as product trust
GPU marketplace / LLM-in-Core
Pack 3 QUEUE invention
```

## 6. Behavior

| Surface | Rule |
|---------|------|
| Settings mock | Work readiness fail-closed |
| Settings process + model | Host LLM gate OK; still needs tip/catalog for Auto/Required |
| Calculate 2+2 text | Treated as generate; needs host LLM (no ready escape) |
| C1 pipeline | Process `Executed`; must not mint VRA |
| Alpha DoD | Same process smoke + C0/C1 green |

## 7. Acceptance

1. Unit: mock settings → Work not ready even for math text.
2. Unit: process settings + available catalog → Compare/Required can be ready.
3. C1/C0/alpha: no mandatory `result=4` / VERIFIED math path.
4. Docs/help state legacy OP-001 and host LLM ≥1 for Work.

## 8. References

- RFC-0241 (Settings Ollama bind), RFC-0232 (work readiness), RFC-0110 (process backend)
- Manifesto OP-001 / Issue #53 — historical only after this RFC
- `docs/desktop-ux.md`, `docs/implementation-status.md`, Analyze-394
