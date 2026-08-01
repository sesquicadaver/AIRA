# Living Spec Matrix — Analyze-63

| ТЗ | Модуль | Тести |
|----|--------|-------|
| rotate same publisher | `rotate_csu_tenant_signing` | `rotate_happy_path_and_audit` |
| rotate missing / backup archive | `rotate_csu_tenant_signing` | `rotate_refuses_missing_and_backup_archives` |
| revoke + audit | `revoke_csu_tenant_signing` | `revoke_removes_dir_map_and_audits` |
| never drop primary | `unregister_verifying` | `revoke_never_drops_primary_signer` |
| refuse overwrite / force | `save_csu_tenant_signing` | `register_default_refuses_overwrite_and_force_allows` |
| one publisher per CSU | `register_csu_tenant_signing` | `register_refuses_duplicate_publisher` |
| secret-before-meta | `commit_secret_then_meta` | `save_secret_first_partial_commit_fail_closed` |
| CLI | `identity csu-tenant rotate\|revoke` | UltraQA smoke |
