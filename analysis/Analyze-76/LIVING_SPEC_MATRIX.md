# Living Spec Matrix — Analyze-76

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Canonical Artifact sign | `ArtifactDescriptor::attach_canonical_signature` | `descriptor_for` + publish |
| Canonical Artifact verify | `CasArtifactStore::publish` | `cas_publish_resolve_and_hash_mismatch` |
| Tenant Artifact sign | `attach_canonical_signature_for_tenant` | `publisher_override_signs_distinct_from_primary` |
| Field mutation breaks verify | same | `canonical_verify_fails_when_artifact_fields_change` |
| Payload vs claimed hash | publish after canonical | HashMismatch case |
