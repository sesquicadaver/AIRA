# Living Specification — Analyze-100 / QUEUE #65

| ТЗ | Модуль | Тести |
|----|--------|-------|
| RFC ShareOffer | `specs/rfc/AIRA-RFC-0014-model-share-offer-payload-schema.md` | docs |
| Schema `$id` | `schemas/model/share-offer.schema.json` | load + validate |
| Valid fixture visibility=local | `fixtures/valid/model/share-offer.json` | manifest valid |
| Invalid missing visibility | `fixtures/invalid/model/share-offer-missing-visibility.json` | manifest invalid |
| Registry unit | `crates/aira-schema` | `model_share_offer_payload_schema_loads` |
