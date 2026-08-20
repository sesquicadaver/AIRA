# Living Specification — Analyze-101 / QUEUE #66

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC share gate | `specs/rfc/AIRA-RFC-0015-share-custom-models-gate.md` | docs |
| no policy → DENY | `request_publish` | `publish_deny_without_policy` |
| share false → DENY | same | `publish_deny_when_share_false` |
| share true → ALLOW, no offer | same | `publish_allow_when_share_true_no_offer_bytes` |
| CLI publish / policy set | `aira-cli` models | smoke |
