# Правила Analyze-17

Partial Local C2 protocol surface.

## Scope
Issue #71–#75 only.

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Envelope/response/identity must match Schema Pack `$id`s
3. Discovery returns Capability + provider CSU — never Node
4. Unsigned envelope fixtures must fail schema validation
5. Local-only adapters (no network)

## Out of scope
Federation (#FED), settlement, Epic 11 docs/security (#76+)
