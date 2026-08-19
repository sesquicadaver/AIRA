# Provenance — CLI / Local Node scope

## Sources
- `Manifesto etc/AIRA Initial Issue Set v0.1.md` Epic 8 (#57–#62)
- `Manifesto etc/AIRA Repository Bootstrap Plan v0.1.md` §10 Local Node State Layout

## Mapping
| Layout (bootstrap) | MVP path |
|--------------------|----------|
| config.yaml | `config.json` (same fields) |
| identity/ | `identity/local.identity.json` + `local.ed25519` |
| db/aira.sqlite | `db/aira.sqlite` (schema via SqliteObjectStore) |
| artifacts/sha256 | `artifacts/` + `index.json` |
| csu/registry | `csu/registry.json` |
| events/ | `events/event-log.json` |

## Decisions
- JSON config for MVP (no serde_yaml dependency yet)
- `run_nonce` namespaces CSU-generated ids for multi-submit safety
- Operational plane still in-process; CLI persists events/problems/artifacts
