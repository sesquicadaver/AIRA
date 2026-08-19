# Правила Analyze-13

Immutability + MVP freeze + safe execution only.

## Scope
Issue #41–#46 only (Epic 6 Basic CSU Set).

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. No shell / network in execution-basic
3. Output Artifact ≠ Verified Result Artifact
4. CSU isolation via `CsuExecutionContext` only

## Out of scope
Epic 7 end-to-end operational flow (#47+).
