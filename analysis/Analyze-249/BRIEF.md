# BRIEF — Analyze-249

Fail-close generate-local / `execution-llm` unless a Phase D activated model is present. Inject `ModelActivateGate` (same pattern as `GenerateBackend`); CSU ↛ CSU. No activate → CapsuleFailed + Evidence, not VERIFIED. Activated test double + MockBackend still CapsuleCompleted. C1 `Calculate 2 + 2` stays `execution-basic`. Do not implement process backend (`#215`) or file RFC-0104.
