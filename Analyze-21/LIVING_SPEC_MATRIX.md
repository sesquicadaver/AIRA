# Living Spec Matrix — Analyze-21

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Keyring load node identity | `aira-object::Keyring` | `node_identity_keyring_sign_verify` |
| Process verify resolves key_ref | `verify_ed25519` | crypto + flow session test |
| LocalSession installs keyring | `LocalSession::open` | `local_session_registers_node_identity` |
| CLI sign/verify | `aira identity sign\|verify` | manual smoke + create registers |
| Docs | `docs/crypto.md` | — |
| Immutability | soft-gates | `deny-originals.sh` |
