# VERIFICATION — Analyze-48

```text
cargo test -p aira-node
# 18 passed (incl. http_bearer_* + health exempt)

cargo clippy -p aira-node -- -D warnings
# clean
```

CLI surface:
- `--http-token` / `AIRA_HTTP_TOKEN` (clap env feature)
- requires `--http`; empty token rejected
