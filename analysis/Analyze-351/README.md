# Analyze-351 — Result-by-problem authority

**QUEUE:** `#315` · **RFC-D:** [AIRA-RFC-0200](../../specs/rfc/AIRA-RFC-0200-result-by-problem-authority.md)

## Outcome

`get_result` by problem id resolves through `verified_artifact_id` / `execution_artifact_id` and ArtifactStore integrity checks. Mutating only `ProblemRecord.result` does not change the returned body; a result cache without an artifact locator fails closed.
