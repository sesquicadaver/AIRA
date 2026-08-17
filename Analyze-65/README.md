# Analyze-65 — YAML config read parity (QUEUE #30)

## Scope
Read `config.yaml` **xor** `config.json` into the same `NodeConfig`. `aira init` still writes JSON only.

## Done when
- Equivalence test yaml ≡ json default
- Both files → fail-closed
- YAML-only open / status / init-idempotent
- Docs + QUEUE + Living Spec

## Out
YAML write; convert CLI; SQLite audit; hot-reload.
