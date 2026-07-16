# Правила Analyze-14

Immutability + reuse-before-compute + no auto normative collapse.

## Scope
Issue #47–#56 only (Epic 7 Operational Flow).

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Pipeline uses Event drain + Active CSU dispatch with ArtifactStore
3. Failures become evidence; Verified Result only after verification
4. Normative split → human collapse stub (no auto-collapse)

## Out of scope
Full CLI problem/result commands (#57–#62), conformance runners.
