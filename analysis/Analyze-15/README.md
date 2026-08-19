# Analyze-15 — Epic 8 CLI / Local Node

**Scope:** Issue Set #57–#62

## Ralplan (approved)

### Principles
1. `aira init` creates `.aira/` local node layout (config, db, artifacts, identity, csu, events)
2. Identity: Ed25519 keypair + local identity descriptor on disk
3. CLI drives `OperationalPlane` for problem → verified result
4. Read commands use persisted events/artifact index/object store
5. `aira-node` loads config, lists CSU, can process a local problem once

### Decision
JSON config (`config.json`) for MVP (same fields as bootstrap YAML). Persist artifact index + event log + problem index under `.aira/`. Keep existing `aira csu list|register` under `.aira/csu/registry.json`.

### Acceptance
- #57 `aira init` → `.aira/`, SQLite, artifacts/, config
- #58 `aira identity create` → keypair + identity JSON
- #59 `aira csu list|register` (path under `.aira`)
- #60 `aira problem submit --text` + `status`
- #61 `result get`, `artifact get`, `event tail`
- #62 `aira-node` starts, loads config/CSU, can run local process
- cargo test/fmt/clippy PASS; originals untouched

### Out of scope
Epic 9 conformance runners (#63+), HTTP API, YAML config parser.
