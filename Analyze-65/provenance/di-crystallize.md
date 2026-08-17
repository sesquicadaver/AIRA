# DI crystallize — Analyze-65 / QUEUE #30

**Chosen:** **A** (user APPROVE)
- Read `config.yaml` **or** `config.json` into same `NodeConfig`
- Both present → fail-closed
- `init` writes only `config.json`
- Initialized iff either file exists
- Out: YAML write, SQLite audit, hot-reload
