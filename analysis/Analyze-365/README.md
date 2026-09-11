# Analyze-365 — Capsule↔Output↔Result binding

**QUEUE:** `#328` · **RFC-0213**

## Outcome

Generate-local ExecutionArtifact stamps `ExecutorFacts` (`model_ref` + `model_content_hash`) from the activate gate. CapsuleCompleted binds `[output, capsule]`. Admission `model_ref` flows into capsule `model_artifact_ref`.
