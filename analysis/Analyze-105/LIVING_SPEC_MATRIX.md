# Living Specification — Analyze-105 / QUEUE #70

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC publish | `specs/rfc/AIRA-RFC-0019-local-rating-evidence-publish.md` | docs |
| missing context | `publish_rating` | `publish_requires_context` |
| CAS + pointer + Event | same | `publish_writes_artifact_pointer_event` |
| network=none | `rating_manifest` | `manifest_network_none` |
| dep firewall | workspace | `dep_firewall.py` |
