# LIVING_SPEC_MATRIX — Analyze-311 / QUEUE #276

| Requirement | Module | Test | Status |
|-------------|--------|------|--------|
| Unique install ID | `new_desktop_identity_id` / `ensure_local_identity` | `two_roots_get_distinct_identity_ids_via_ensure_bootstrap` | **DONE** |
| No silent legacy migrate | early return if identity files exist | `legacy_identity_file_is_not_silently_migrated` | **DONE** |
| TrustStore key collision | `TrustStore::upsert` | `trust_upsert_rejects_different_key_for_same_id` | **DONE** |
| RFC-D | `AIRA-RFC-0165` | file; RFC-0164 free | **DONE** |
| QUEUE advance | `#276` DONE → `#277` | `phase_q_doc` | **DONE** |
