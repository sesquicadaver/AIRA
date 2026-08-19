# Living Spec — Analyze-71 / QUEUE #36

| ТЗ | Модуль | Тести |
|----|--------|-------|
| List tenant backup slots | `list_csu_tenant_secret_backups` + CLI `identity csu-tenant backups` | list columns; latest present after rotate `--backup` |
| Prune archived stamps only | `prune_csu_tenant_secret_backups` + CLI `… prune` | keep=1 two tenants; keep=0 keeps latest; dry-run; no flags fail |
| Per-tenant retain ∩ | same | two-tenant isolation of `--keep` |
| Never latest / live secret | same | latest + `ed25519` survive |
| Orphan meta / unparseable age | same | skip, not delete |
| Node prune unchanged | `identity backups prune` | tenant files untouched |
| Numeric rank `9` vs `10` | `stamp_sort_key` | `prune_numeric_rank_prefers_10_over_9` |
| Skip `.tmp` staging | `archived_prev_stamp` | `prune_and_list_ignore_tmp_staging` |

**Honest:** local tenant archive GC. Not a unified backups command. Not stdin secret import (#37).
