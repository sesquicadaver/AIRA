# Living Spec Matrix — Analyze-82

| ТЗ | Модуль | Тести |
|----|--------|-------|
| Mechanical crypto split | `crypto/error.rs` constants+errors | `crypto::tests::*` via `mod.rs` |
| Keyring / sign / verify | `crypto/keyring.rs` | local_test + node identity tests |
| Trust store + CRL | `crypto/trust_store.rs` (EVO revocation) | trust_crl / rotate / rekey tests |
| Node rotate / backup / prune | `crypto/rotation.rs` | node_rotate_* / prune_* |
| tenant.rs untouched | `crates/aira-object/src/tenant.rs` | tenant tests still pass |
