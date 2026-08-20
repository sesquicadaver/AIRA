# Living Specification — Analyze-98 / QUEUE #63

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC verify | `specs/rfc/AIRA-RFC-0012-quarantine-verify.md` | docs |
| hash+sig match → verified/ | `verify_quarantine` | `verify_promotes_to_verified_on_match` |
| hash mismatch → reject, quarantine kept | same | `verify_rejects_hash_mismatch_keeps_quarantine` |
| TESTSIG unsigned → reject | same | `verify_rejects_unsigned_testsig` |
| CLI exit 0/2 | `models verify` | smoke |
