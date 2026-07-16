# Living Spec Matrix — Analyze-30

| ТЗ / вимога | Модуль | Тести |
|-------------|--------|--------|
| Same identity_id, new secret | `rotate_node_signing_secret` | `node_signing_secret_rotate_cutover` |
| Trust upsert, no CRL | `ensure_trust_defaults` | same + process keyring cutover asserts |
| Rollback if node on CRL | `rotate_node_signing_secret` | `node_rotate_rolls_back_when_node_revoked` |
| CLI rotate | `IdentityCommands::Rotate` | CLI smoke in RALPH_EVIDENCE |
| Docs node vs peer | `docs/crypto.md` | checklist |
| Fail if no identity | `rotate_node_signing_secret` | `node_rotate_requires_existing_identity` |
