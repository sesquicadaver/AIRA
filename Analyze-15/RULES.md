# Правила Analyze-15

Local CLI / node surface for Epic 8.

## Scope
Issue #57–#62 only (Epic 8 CLI / Local Node).

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. `aira init` creates the local layout; commands fail clearly if missing
3. Artifact CAS index must persist across process restarts
4. Multi-submit must not collide on artifact/event ids (`run_nonce`)
5. No network/shell for CSU in default config

## Out of scope
Conformance runners (#63+), HTTP API, YAML config parser, federation.
