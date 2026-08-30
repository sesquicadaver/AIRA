# Living spec — Analyze-224 (QUEUE #189)

| ТЗ | Модуль / артефакт | Тест | Статус |
|----|-------------------|------|--------|
| Persistent index | `problems/reuse-index.json` | `local_session_repeat_problem_reuses_without_execution` | **DONE** |
| Repeat same text | LocalSession submit | no `CapsuleCompleted`; `reuse:ready_solution` | **DONE** |
| Survive reopen | second `LocalSession::open` | same test, third submit | **DONE** |
| Different text | `echo hello` after 2+2 | still executes | **DONE** |
| RFC | `AIRA-RFC-0087-durable-reuse-index.md` | `phase_i_durable_reuse_189` | **DONE** |
| Fail-closed signing | `active_signature` | — | **OUT** (`#190`) |
