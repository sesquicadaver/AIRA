# Living Spec Matrix — Analyze-22

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Primary signer API | `aira-object` | `node_identity_keyring_sign_verify` |
| support local_* → active | `aira-csu::support` | flow/csu suites |
| LocalSession register before plane | `local.rs` | `local_session_submit_signs_with_node_identity` |
| ObjectDescriptor over content_hash | `plane.rs` | same |
| Docs | `docs/crypto.md` | — |
| Immutability | soft-gates | `deny-originals.sh` |
