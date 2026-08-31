# Living spec — Analyze-236 (QUEUE #201)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| mint not default prelude | `crates/aira-object/src/lib.rs` cfg `store-backend` | `object_store_access_is_not_in_the_default_prelude` | **DONE** |
| store-backend only aira-core | `crates/aira-core/Cargo.toml` | `store_backend_feature_is_only_enabled_by_aira_core` | **DONE** |
| CSU no mint import | `csu/**/*.rs` | `csu_sources_do_not_import_object_store_access` | **DONE** |
| C0 opacity without mint | `crates/aira-conformance/src/c0.rs` | `c0.object.handle_opacity` | **DONE** |
| RFC-0097 | `specs/rfc/AIRA-RFC-0097-seal-object-store-access.md` | `phase_j_seal_object_store_access_201` | **DONE** |
| VRA payload | C1 2+2 body | — | **OUT** (`#202`) |
