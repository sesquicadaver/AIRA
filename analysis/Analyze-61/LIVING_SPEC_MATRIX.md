# Living Spec Matrix — Analyze-61

| ТЗ | Модуль | Тести |
|----|--------|-------|
| keep-N archives | `prune_node_secret_backups` | `prune_keep_one_retains_newest_archive_and_latest` |
| TTL days | same | `prune_older_than_days_skips_unparseable_when_ttl_set` |
| dry-run | same | `prune_dry_run_deletes_nothing` |
| orphan meta | same | `prune_never_deletes_orphan_meta` |
| x25519 | `prune_noise_static_backups` | `prune_noise_static_keep_zero_drops_archives_keeps_latest` |
| CLI | `identity backups prune` | UltraQA smoke |
