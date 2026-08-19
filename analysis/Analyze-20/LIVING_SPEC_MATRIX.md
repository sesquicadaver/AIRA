# Living Spec Matrix — Analyze-20

| ТЗ / TODO | Модуль | Тести |
|-----------|--------|-------|
| Real Ed25519 sign/verify | `aira-object::crypto` | `crypto::tests::*` |
| Artifact admission | `CasArtifactStore::publish` | artifact + `sec.invalid_artifact_signature` |
| Event admission | `MemoryEventLog::append` | event tests |
| CSU admission | `CsuManifest::validate_for_registration` | csu + `sec.invalid_csu_signature` |
| Protocol envelope | `ProtocolEnvelope::validate_signature` | protocol envelope test |
| Invariant signature | `InvariantChecker::check_event_signature` | core invariants |
| Docs | `docs/crypto.md` | — |
| Immutability | soft-gates | `deny-originals.sh` |
