# VERIFICATION — Analyze-51

```text
cargo test -p aira-node
# 24 passed (mtls_accepts/rejects_*, bearer coexistence, A-45/A-48 regression)

cargo clippy -p aira-node -- -D warnings
# clean
```

CLI: `--tls-client-ca` requires HTTPS; empty CA fail-closed; startup warning about `/health`.
