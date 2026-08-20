# AIRA-RFC-0020 — CLI `aira models rate` (RFC-E)

## 1. Summary

`aira models rate` publishes contextual ModelRatingEvidence via `publish_rating` (CSU `#70`). Required: `--model-ref`, `--context-id`, `--task_class`, `--reason`. Optional dimensional scores. Prints `global_score false` / `network false`.

## 2. Non-Goals

```text
upgrade recommend (D7)
marketplace / global ranking
federation
```

## 3. Evidence

[`docs/phase-d-plan.md`](../../docs/phase-d-plan.md) §6c D6.3; QUEUE `#71`.
