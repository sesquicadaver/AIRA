# Living Spec Matrix — Analyze-79

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Envelope verify без domain fallback | `ProtocolEnvelope::validate_signature` | `envelope_rejects_local_test_domain_fallback` |
| Envelope production sign = payload_hash | `signature_over_payload_hash`; EP/AP `wrap*` | event/artifact protocol publish |
| Identity verify = identity_id bytes | `IdentityDescriptor::local_user` | `identity_rejects_local_test_domain_signature` |
| Schema fixtures ≠ live crypto | protocol envelope fixture + live re-sign | `envelope_schema_valid_and_unsigned_rejected` |
