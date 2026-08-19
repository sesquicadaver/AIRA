# Правила Analyze-28

Dual-key grace window.

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. `--until` must be RFC3339 UTC (`…Z`); invalid → error
3. Grace never allows upsert of CRL ids
4. Document in `docs/crypto.md`
