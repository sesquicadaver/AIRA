# Ralplan — Analyze-65 / QUEUE #30 (rev1)

## Principles
1. Same `NodeConfig` schema; format is transport only.
2. Fail closed if both `config.yaml` and `config.json` exist.
3. `init` remains JSON-only writer; idempotent on YAML-only nodes.
4. Maintained YAML dep: **`serde_norway`** (fork of archived `serde_yaml`; not `serde_yml`).

## Scope
1. `NodePaths::config_json()` / `config_yaml()`; `config()` alias → json (writers).
2. `node_config_present(root)` → json XOR yaml exists (not both).
3. `load_config`: both → Err; yaml → `serde_norway::from_str`; json → json; neither → Err.
4. `init_node`: write json **only if `!node_config_present`** (YAML-only node stays untouched).
5. Presence gates: `LocalSession::open`, CLI `ensure_init`, CLI `Status` (`main.rs:501`), `aira-node` init check — all use `node_config_present`.
6. Docs + QUEUE + Living Spec. Conformance alpha keeps expecting `config.json` after `init_node` (json writer).

## Out
YAML write; JSON↔YAML convert CLI; SQLite audit; hot-reload.

## Tests
- `load_config_yaml_matches_json`
- `load_config_both_fail_closed`
- `load_config_json_only_ok`
- `open_accepts_yaml_only_node`
- `init_writes_json_not_yaml`
- `init_idempotent_on_yaml_only_node` (no extra json created)
- `status_accepts_yaml_only_node` (via `node_config_present` / open path)
