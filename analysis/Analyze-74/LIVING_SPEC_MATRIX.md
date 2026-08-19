# Living Spec Matrix — Analyze-74

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Canonical JSON + SHA-256 | `canonical_json_bytes` / `descriptor_signing_hash` | `key_order_and_whitespace_do_not_change_hash` |
| Strip top-level signature | `strip_top_level_signature` | `top_level_signature_is_stripped_from_hash` |
| Nested signature stays | canonicalize recurse | `nested_signature_field_is_not_stripped` |
| Sign/verify over hash string | `sign_canonical_descriptor` | `sign_verify_roundtrip_and_reject_mutation` |
| Not payload_hash-only | helper vs `verify_ed25519` | `helper_does_not_accept_payload_hash_only_message` |
| Runtime Event path unchanged | `invariants.rs` fallback intact | `existing_payload_hash_verify_path_still_independent` |
