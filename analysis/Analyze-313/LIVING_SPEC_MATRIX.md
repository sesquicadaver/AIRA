# Living Spec Matrix — Analyze-313 / QUEUE `#278`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| Root-scoped verification for observe | `activate_gate.rs` `verification_keyring` + `Keyring::verify` | `observe_ready_with_disk_identity_without_process_keyring_priming` |
| Reopen without prior test keyring → ready | disk identity + evidence; no process priming | `observe_ready_after_reopen_without_process_keyring_priming` (child process) |
| Separate process test | same test re-exec via `current_exe` | child branch under `AIRA_278_REOPEN_CHILD` |
| RFC-0167 | `specs/rfc/AIRA-RFC-0167-model-verify-context.md` | `phase_q_rfc_0167_present` |
