# Living Specification — Analyze-102 / QUEUE #67

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC local publish | `specs/rfc/AIRA-RFC-0016-local-publish-signed-descriptor.md` | docs |
| DENY → no ShareOffer | `publish_local` | `publish_local_deny_without_policy` |
| ALLOW без cache → error | same | `publish_local_requires_activated_cache` |
| bad visibility | same | `publish_local_rejects_bad_visibility` |
| ALLOW + cache → CAS + pointer + Event | same | `publish_local_writes_signed_descriptors_from_cache` |
| Gate alone без offer | `request_publish` | `publish_allow_when_share_true_no_offer_bytes` |
| CLI publish flags | `aira-cli` models | clap + smoke |
