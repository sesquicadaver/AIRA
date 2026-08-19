# Living Spec Matrix — Analyze-11

| ID | Issue | Реалізація | Перевірка | Статус |
|----|-------|------------|-----------|--------|
| LS11-001 | #27 | `ArtifactDescriptor` | schema validate test | mapped |
| LS11-002 | #28 | `CasArtifactStore` | publish/resolve + hash mismatch | mapped |
| LS11-003 | #29 | `supersede` | old payload retained | mapped |
| LS11-004 | #30 | `EventDescriptor` | schema validate test | mapped |
| LS11-005 | #31 | `MemoryEventLog` + `EventSink` | mutate fails; query by refs | mapped |
| LS11-006 | #32 | `subscribe` + idempotent append | single delivery on duplicate id | mapped |
| LS11-007 | #33 | `PolicyGate` | ALLOW/DENY/REQUIRE + PolicyEvaluated | mapped |
| LS11-008 | #34 | `InvariantChecker` | PolicyDenied → InvariantViolation event | mapped |
| LS11-009 | Epic 5 | Analyze-12 | done → [../Analyze-12/](../Analyze-12/) | mapped |
