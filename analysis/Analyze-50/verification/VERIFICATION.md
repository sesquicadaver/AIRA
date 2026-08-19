# VERIFICATION — Analyze-50

```text
cargo test -p aira-object   # incl. trust_rekey_grace_allows_old_same_id
cargo test -p aira-peer     # incl. notify_rekey_with_grace_keeps_old_pubkey
cargo clippy -p aira-object -p aira-peer -- -D warnings
```
