# Living Specification — Analyze-97 / QUEUE #62

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC local quarantine | `specs/rfc/AIRA-RFC-0011-quarantine-local-fetch.md` | docs |
| ALLOW + local source → quarantine | `fetch_to_quarantine` | `quarantine_fetch_after_allow_copies_local_source` |
| DENY → no copy | same | `quarantine_denied_without_policy_no_copy` |
| Reject HTTP source | same | `quarantine_rejects_http_source` |
| Gate-only without `--source` | CLI | smoke |
| No verify/activate | receipt flags + Out | unit asserts |
