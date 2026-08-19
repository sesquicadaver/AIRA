# LIVING_SPEC_MATRIX — Analyze-58

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Persist + reload | `RelayHubRegistry` | `registry_survives_restart_and_marks_offline` |
| Offline on disconnect | `mark_offline` | same |
| TTL offline-only | `prune_offline_older_than` | `ttl_prunes_stale_offline_only` |
| TTL none retains | `with_relay_hub_registry(None)` | `ttl_none_retains_stale_offline` |
| Fail-closed schema | `load` | `registry_rejects_bad_schema` |
| Live deliver still needs register | existing | `relay_hub_delivers_trust_delta_a_to_c_via_r` |
| CLI TTL requires --relay | aira-cli | UltraQA |
| Docs / QUEUE #23 | peer-link, QUEUE | manual |
