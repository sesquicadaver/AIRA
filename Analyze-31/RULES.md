# Правила Analyze-31

Opt-in durable backup of previous node signing secret.

## Hard rules
1. Do not edit `Manifesto etc/**` or `Meditation_About/**`
2. Without `--backup`, do not leave durable old secret on disk
3. With `--backup`, fail closed before overwrite if backup cannot be written
4. Do not implement dual-key same-ref or peer notify in this slice
5. Document in `docs/crypto.md`
