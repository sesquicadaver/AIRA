# Living Spec Matrix — Analyze-364 / QUEUE `#327`

| ТЗ / Done when | Модуль | Тести |
|----------------|--------|-------|
| post-copy == VerifiedPointer.content_hash | `activate.rs` | `activate_copies_*` |
| tamper → reject | `activate.rs` / error | `activate_rejects_tampered_*` |
| forged pointer hash → reject | `activate.rs` | `activate_rejects_forged_*` |
| RFC-0212 | specs/rfc | `phase_v_doc` |
