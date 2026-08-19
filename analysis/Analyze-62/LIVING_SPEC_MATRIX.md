# Living Spec Matrix — Analyze-62

| ТЗ | Модуль | Тести |
|----|--------|-------|
| save/load durable | `tenant::{save,load}_csu_tenant_signing` | `save_load_survives_reset` |
| load_all + isolation | `load_all_csu_tenant_signing` | `load_all_isolation_and_empty_ok` |
| meta mismatch fail-closed | `load_csu_tenant_signing` | `meta_pubkey_mismatch_fails_closed` |
| rehydrate after trust sync | `local.rs` + `load_all` | `trust_sync_then_load_all_restores_verifier` |
| CLI | `identity csu-tenant` | UltraQA smoke |
