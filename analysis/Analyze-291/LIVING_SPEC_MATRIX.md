# Analyze-291 — Living Spec Matrix

| ТЗ / atom | Модуль / артефакт | Тест | Статус |
|-----------|-------------------|------|--------|
| UNKNOWN ≠ OFFLINE banner | `MeshTopLevel::from_reachability` | `unknown_reachability_is_not_offline_banner` | **DONE** |
| LOCAL_ONLY ≠ OFFLINE | `NetworkMeshSnapshot.top_level` | `snapshot_reads_port_reachability_peers` | **DONE** |
| AddressBook ≠ live sessions | `address_book_count` / `live_session_count` | `address_book_peers_are_not_live_sessions` | **DONE** |
| SystemSnapshot projection | `load_system_snapshot` / `DataQuality` | unit + `phase_o_*` | **DONE** |
| RFC-D | `AIRA-RFC-0147-…` | `phase_o_rfc_0147_present` | **DONE** |
