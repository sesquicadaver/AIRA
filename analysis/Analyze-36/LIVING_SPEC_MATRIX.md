# Living Spec Matrix — Analyze-36

| ТЗ / Acceptance | Модуль | Тест / доказ |
|-----------------|--------|--------------|
| Build/parse trust-delta | `trust_delta` | `trust_delta_bad_schema_rejected` |
| Encrypted revoke roundtrip + apply | `session` + `trust_delta` | `trust_delta_revoke_roundtrip_applies` |
| Refuse local-test / local node | `apply_trust_delta` | `trust_delta_refuses_local_test_and_local_node` |
| Rotate apply via TrustStore | `apply_trust_delta` | `trust_delta_rotate_shape_and_apply` |
| CLI trust-send / --apply-trust | `aira-cli` | clippy build + docs |
