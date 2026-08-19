# Living Spec Matrix — Analyze-31

| ТЗ / вимога | Модуль | Тести |
|-------------|--------|--------|
| Opt-in backup after successful rotate | `rotate_node_signing_secret(..., backup)` | `node_rotate_backup_writes_prev` |
| No backup by default | same | `node_signing_secret_rotate_cutover` |
| Stage fail closed | same | `node_rotate_backup_fail_closed` |
| Preserve prior `.prev` on trust fail | same | `node_rotate_backup_preserves_prev_slot_on_trust_fail` |
| Commit clears dir trap; no trust≠secret | same | `node_rotate_backup_commit_clears_prev_dir_trap` |
| CLI `--backup` | `IdentityCommands::Rotate` | smoke in RALPH_EVIDENCE |
| Docs | `docs/crypto.md` | checklist |
