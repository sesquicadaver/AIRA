# Living Spec Matrix — Analyze-70

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Local join+trust | `aira-protocol::join_federation` | `join_pins_trust_and_membership` |
| Self-signed TOFU | `verify_federation_descriptor` + `Keyring::with_verifying_hex` | unsigned / invalid hex / `with_verifying_hex_detached_roundtrip_and_no_local_test` |
| CRL fail-closed | join wrapper | `revoked_identity_fails` |
| One membership | join wrapper | `other_federation_id_fail_closed` |
| Same id+key idempotent | join wrapper | `join_pins_trust_and_membership` |
| Different key fail-closed | join wrapper (upsert unchanged) | `same_federation_different_key_fails`; `truststore_pubkey_mismatch_fails` |
| Refuse local-test | join wrapper | `refuses_local_test_identity` |
| Honest TOFU | docs/peer-link.md | other members Untrusted until `trust add` |
| Out: no Join Request / CRP / leave / peer message | CLI + module | no new peer types |
