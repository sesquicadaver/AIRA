# LIVING_SPEC_MATRIX — Analyze-52

| ТЗ / рішення | Модуль | Тести |
|--------------|--------|-------|
| subject==issuer all ops | `trust_delta::apply_trust_delta` | `trust_delta_ops_require_issuer_subject_match` |
| Self-revoke over peer | `session` + `trust_delta` | `trust_delta_revoke_roundtrip_applies` |
| Self-rotate apply | `trust_delta` | `trust_delta_rotate_shape_and_apply` |
| Protected local-test/local node | `trust_delta` | `trust_delta_refuses_local_test_and_local_node` |
| Gossip self-announce | gossip + apply | `gossip_trust_delta_a_to_b_to_c` |
| Relay self-announce | relay + apply | `relay_hub_delivers_trust_delta_a_to_c_via_r` |
| Send gate subject==local | `make_trust_delta_envelope` | covered by roundtrip / IdentityMismatch on third-party send |
| Docs | `docs/peer-link.md` | manual |
| QUEUE #17 | `QUEUE.md` | — |
| A-36 policy TODO | `Analyze-36/todo/TODO_FIXME.md` | closed → A-52 |
