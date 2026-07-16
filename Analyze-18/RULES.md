# Правила Analyze-18

MVP Alpha release surface.

## Scope
Issue #76–#80 only.

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Docs must match runnable CLI/library paths
3. Security: unsigned CSU/artifact rejected; private default-deny; no secrets in events
4. Release pack must not require network/ML/GPU/blockchain
5. Do not commit built binaries under `release/`

## Out of scope
GitHub Release upload automation, production crypto, federation
