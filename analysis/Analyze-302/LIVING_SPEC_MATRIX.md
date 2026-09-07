# LIVING_SPEC_MATRIX — Analyze-302 / QUEUE #267

| Requirement | Module | Test | Status |
|-------------|--------|------|--------|
| measurement ≠ load | `SystemSnapshot::{observed_at,loaded_at}` | `fresh_checked_at_is_current_and_differs_from_loaded_at` | **DONE** |
| Stale path | `classify_network_quality` | `old_checked_at_is_stale` | **DONE** |
| missing checked_at → Unknown | `classify_network_quality` | `missing_checked_at_is_unknown_not_current` | **DONE** |
| bind ≠ listener | `LocalBindProvenance` + `local_listener_proven` | `reachability_port_without_config_is_not_listener_proof` | **DONE** |
| strip honesty | `ui_status_strip` + quality | compile + unit (runtime) | **DONE** |
| RFC-D | `AIRA-RFC-0157` | file present; RFC-0156 still free | **DONE** |
| QUEUE advance | `#267` DONE → `#268` OPEN | `phase_p_doc` | **DONE** |
