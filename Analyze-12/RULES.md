# Правила Analyze-12

Immutability + MVP freeze + CSU isolation.

## Scope
Issue #35–#40 only (Epic 5 CSU Runtime).

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Unsigned manifests rejected; ABI must be `0.1`
3. Invalid lifecycle transitions rejected
4. Only Active CSU receive dispatched events
5. Context denies Core/Artifact mutation and peer CSU calls

## Out of scope
Domain CSU logic (#41+), WASM/subprocess packaging, crypto signature verify.
