# Living Spec Matrix — Analyze-65

| ТЗ | Модуль | Тести |
|----|--------|-------|
| YAML∨JSON → NodeConfig | `aira_flow::load_config` + `serde_norway` | `load_config_yaml_matches_json`, `load_config_json_only_ok` |
| Both files fail-closed | `load_config` | `load_config_both_fail_closed` |
| Presence gate | `node_config_present` | `open_accepts_yaml_only_node`, `status_accepts_yaml_only_node` |
| Init JSON-only writer | `init_node` | `init_writes_json_not_yaml`, `init_idempotent_on_yaml_only_node` |
| CLI/node gates | `aira-cli` Status/`ensure_init`, `aira-node` | covered via `node_config_present` |
