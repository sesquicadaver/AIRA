# Analyze-285 — Reachability session bind (QUEUE #250)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Noise HS hash on session | `noise.rs` / `AuthenticatedPeer` | `session_bound_probe_sets_proof_and_direct` | **DONE** |
| Session transcript domain | `session_transcript_hex` | same + verify recompute | **DONE** |
| success=true needs session API | `issue_for_authenticated_session` | `signed_claim_without_session_cannot_apply_direct` | **DONE** |
| DIRECT needs local inbound tx | `apply_successful_probe(..., local_tx)` | wrong transcript fails | **DONE** |
| CLI bind | `--session-transcript` | compile + docs | **DONE** |
| RFC-D | `AIRA-RFC-0142-…` | `phase_n_fix_*` | **DONE** |
| Expiry before promote | — | — | **OUT** (`#251`) |
