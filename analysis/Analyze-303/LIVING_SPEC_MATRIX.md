# LIVING_SPEC_MATRIX — Analyze-303 / QUEUE #268

| Requirement | Module | Test | Status |
|-------------|--------|------|--------|
| unconfirmed ≠ Applied | `SettingsApplyPhase::Undefined` | `unconfirmed_is_undefined_not_applied` | **DONE** |
| Start confirms applied | `from_start_outcome` | `start_outcome_uses_outcome_listens` | **DONE** |
| status Running confirms | `from_status` | `running_http_only_confirms_p0` / peer | **DONE** |
| Stop clears applied | `clear_applied_runtime` | compile + unit | **DONE** |
| GUI open no fake Applied | `applied_runtime: None` at init | phase Undefined until refresh | **DONE** |
| RFC-D | `AIRA-RFC-0158` | file present; RFC-0156 free | **DONE** |
| QUEUE advance | `#268` DONE → `#269` OPEN | `phase_p_doc` | **DONE** |
