# Living Spec Matrix — Analyze-75

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Canonical Event sign | `EventDescriptor::attach_canonical_signature` | `aira-event` sample + schema |
| Canonical Event verify, no domain fallback | `verify_canonical` / `MemoryEventLog::append` / `InvariantChecker::check_event_signature` | `canonical_verify_fails_when_event_type_or_causal_refs_change` |
| Tenant Event sign | `attach_canonical_signature_for_tenant` / `make_event_as` | `emit_failed_and_lifecycle_use_publisher_identity` |
| Field mutation breaks verify | same | mutation test (type/refs/hash) |
| Secret filter after valid signature | `payload_contains_secret` | `secret_material_rejected_in_payload_ref`; `sec.no_secrets_in_events` |
