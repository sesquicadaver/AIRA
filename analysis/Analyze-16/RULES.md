# Правила Analyze-16

Conformance C0/C1 for Epic 9.

## Scope
Issue #63–#70 only.

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Report must validate against `aira:schema:conformance:report:0.1`
3. Report published as immutable `ConformanceArtifact`
4. C0/C1 runners must fail closed (failed count > 0 → non-zero CLI exit)
5. No network/shell in suite

## Out of scope
Epic 10 protocols (#71+), demo docs (#76), C2–C5 profiles
