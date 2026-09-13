# Living Spec — Analyze-375 / `#338`

| ТЗ | Модуль | Тести |
|----|--------|-------|
| no-follow materialize | `csu/model-acquisition/src/materialize.rs` | `materialize::*`; `activate_rejects_*_symlink*`; `verify_rejects_*`; `quarantine_rejects_*` |
| post-copy hash | `materialize_weights_nofollow` | `materialize_expected_mismatch_removes_dest`; activate hash mismatch |
| bounded buffer | `WEIGHTS_IO_BUF` | stream unit tests; no `fs::read` full weights on materialize path |
| RFC-0222 | `specs/rfc/AIRA-RFC-0222-safe-weights-materialization.md` | `phase_w_doc.rs` |
