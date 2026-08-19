# Living Spec Matrix — Analyze-83

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Mechanical tenant split | `tenant/mod.rs` re-exports | `tenant::tests::*` |
| Paths + CSU hex dir names | `tenant/paths.rs` | `encode_decode_roundtrip` |
| In-memory map / isolation | `tenant/map.rs` | isolation / duplicate publisher |
| Durable save/load/list | `tenant/persist.rs` | save_load / load_all / meta mismatch |
| Rotate / revoke ceremony | `tenant/ceremony.rs` | rotate_* / revoke_* |
| Backup list + prune | `tenant/prune.rs` | prune_* / list_includes_latest |
| crypto.rs untouched | `crates/aira-object/src/crypto/` | crypto tests still pass |
